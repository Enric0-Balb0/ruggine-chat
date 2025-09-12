use crate::dto::invitation_dto::{InvitationUpdateStatusDto, InvitationUpdateResponseDto};
use crate::error::{api_error::ApiError, request_error::ValidatedRequest};
use crate::response::api_response::ApiSuccessResponse;
use crate::state::invitation_state::InvitationState;
use crate::entity::user::User;
use axum::{extract::State, Extension, Json};
use tracing::warn;
use crate::entity::invitation::InvitationStatus;

#[utoipa::path(
    patch,
    path = "/api/invitation/update-status",
    request_body = InvitationUpdateStatusDto,
    responses(
        (status = 200, description = "Invitation status updated successfully", body = ApiSuccessResponseInvitationUpdateResponseDto),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 403, description = "Forbidden - User is not the recipient of the invitation"),
        (status = 404, description = "Invitation not found"),
        (status = 409, description = "Invitation already responded"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Invitation",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_status(
    Extension(current_user): Extension<User>,
    State(state): State<InvitationState>,
    ValidatedRequest(payload): ValidatedRequest<InvitationUpdateStatusDto>,
) -> Result<Json<ApiSuccessResponse<InvitationUpdateResponseDto>>, ApiError> {
    let invitation_response = state.invitation_service.update_status(payload, current_user.id).await?;

    // Invia notifica WebSocket ai membri del gruppo se il servizio è disponibile
    if let Some(websocket_service) = &state.websocket_group_service {
        if invitation_response.status == InvitationStatus::Accepted {
            if let Some(group_membership_id) = invitation_response.group_membership_id {
                let group_membership = state.group_membership_service
                    .find_by_id_and_user_id(group_membership_id, current_user.id)
                    .await?;

                crate::handler::websocket::chat_handler::handle_new_group_membership(
                    websocket_service.clone(),
                    state.ws_manager.clone().unwrap().clone(),
                    group_membership.group_chat_id,
                    current_user.username.clone(),
                ).await;
            }
        }

    } else {
        warn!("WebSocket group service not available for sending notifications");
    }

    Ok(Json(ApiSuccessResponse::send(invitation_response)))
}
