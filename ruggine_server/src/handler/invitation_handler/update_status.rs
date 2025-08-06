use crate::dto::invitation_dto::{InvitationUpdateStatusDto, InvitationUpdateResponseDto};
use crate::error::{api_error::ApiError, request_error::ValidatedRequest};
use crate::response::api_response::ApiSuccessResponse;
use crate::state::invitation_state::InvitationState;
use crate::entity::user::User;
use axum::{extract::State, Extension, Json};

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
    Ok(Json(ApiSuccessResponse::send(invitation_response)))
}
