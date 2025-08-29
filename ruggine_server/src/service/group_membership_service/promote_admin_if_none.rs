use tracing::error;
use crate::error::{api_error::ApiError, db_error::DbError, group_membership_error::GroupMembershipError};
use crate::dto::group_membership_dto::{GroupMembershipCreateDto, GroupMembershipReadDto};
use crate::error::group_chat_error::GroupChatError;
use crate::error::invitation_error::InvitationError;
use crate::service::group_membership_service::GroupMembershipService;

impl GroupMembershipService {
    pub async fn promote_admin_if_none_internal(&self, group_chat_id: i32) -> Result<Option<GroupMembershipReadDto>, ApiError> {
        // Verify invitation exists
        match self
            .group_chat_service()
            .find_by_id(group_chat_id)
            .await
        {
            Ok(_) => { },
            Err(ApiError::GroupChatError(GroupChatError::GroupChatNotFound)) => {
                return Err(ApiError::GroupMembershipError(
                    GroupMembershipError::GroupNotFound,
                ))
            },
            Err(e) => {
                return Err(e);
            }
        };

        let membership = match self.group_membership_repo.promote_admin_if_none(group_chat_id).await {
            Ok(group_membership) => group_membership,
            Err(e) => {
                error!("Something went wrong trying to promote admin if none: {:?}", e);
                return Err(ApiError::DbError(DbError::SomethingWentWrong(e.to_string())));
            }
        };

        return match membership {
            Some(group_membership) => { Ok(Some(GroupMembershipReadDto::from(group_membership))) },
            None => { Ok(None) }
        }
    }
}