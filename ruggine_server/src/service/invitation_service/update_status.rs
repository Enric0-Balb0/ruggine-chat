use std::future::Future;
use crate::config::database::DatabaseTrait;
use crate::dto::group_membership_dto::GroupMembershipCreateDto;
use crate::dto::invitation_dto::{InvitationUpdateResponseDto, InvitationUpdateStatusDto};
use crate::entity::invitation::{InvitationStatus, UpdateInvitationStatus};
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::invitation_error::InvitationError;
use crate::service::invitation_service::InvitationService;
use chrono::Utc;

impl InvitationService {
    pub async fn update_status_internal(
        &self,
        payload: InvitationUpdateStatusDto,
        auth_user_id: i32,
    ) -> Result<InvitationUpdateResponseDto, ApiError> {
        async fn do_update(service: &InvitationService, payload: InvitationUpdateStatusDto, auth_user_id: i32) -> Result<InvitationUpdateResponseDto, ApiError> {
            // Validate that the status is either accepted or rejected
            if let Err(e) = payload.validate_status() {
                return Err(ApiError::InvitationError(InvitationError::InvalidStatus(e)));
            }

            // Find the invitation to verify user authorization
            let invitation = match service.invitation_repo.find_by_id(payload.invitation_id).await
            {
                Ok(invitation) => invitation,
                Err(_) => {
                    return Err(ApiError::InvitationError(
                        InvitationError::InvitationNotFound,
                    ))
                }
            };

            // Verify that the current user is the recipient of the invitation
            if invitation.to_user_id != auth_user_id {
                return Err(ApiError::InvitationError(
                    InvitationError::UserNotAuthorized("Not authorized to update".to_string()),
                ));
            }

            // Verify that the invitation is still pending
            if invitation.status != InvitationStatus::Pending {
                return Err(ApiError::InvitationError(
                    InvitationError::InvitationAlreadyResponded,
                ));
            }

            // Create update status entity
            let update_invitation_status = UpdateInvitationStatus {
                invitation_id: payload.invitation_id,
                status: payload.status.clone(),
            };

            // Update the invitation status
            let updated_invitation = match service
                .invitation_repo
                .update_status(payload.invitation_id, update_invitation_status)
                .await
            {
                Ok(invitation) => invitation,
                Err(e) => {
                    return Err(ApiError::DbError(DbError::SomethingWentWrong(
                        e.to_string(),
                    )))
                }
            };

            let mut response = InvitationUpdateResponseDto::from(updated_invitation);

            // If the status is accepted, create a group membership
            if payload.status == InvitationStatus::Accepted {
                let membership_dto = GroupMembershipCreateDto {
                    invitation_id: invitation.id,
                    role: invitation.role_at_join,
                };

                let created_membership = match service
                    .group_membership_service
                    .create_checked(membership_dto, auth_user_id)
                    .await
                {
                    Ok(membership) => membership,
                    Err(e) => {
                        eprintln!("Failed to create group membership: {:?}", e);
                        return Err(e);
                    }
                };

                // Set the group membership ID in the response
                response.set_group_membership_id(created_membership.id);
            }

            Ok(response)
        }

        if self.invitation_repo.db_conn().get_tx_mut().is_some() {
            do_update(self, payload, auth_user_id).await
        } else {
            self.invitation_repo
                .db_conn()
                .with_tx(|_db| do_update(self, payload.clone(), auth_user_id))
                .await
        }
    }


}

