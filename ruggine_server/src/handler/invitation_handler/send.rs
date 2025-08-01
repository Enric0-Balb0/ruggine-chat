use crate::dto::invitation_dto::{InvitationCreateDto, InvitationReadDto};
use crate::error::{api_error::ApiError, request_error::ValidatedRequest};
use crate::response::api_response::ApiSuccessResponse;
use crate::state::invitation_state::InvitationState;
use crate::entity::user::User;
use axum::{extract::State, Extension, Json};

#[utoipa::path(
    post,
    path = "/api/invitation/send",
    request_body = InvitationCreateDto,
    responses(
        (status = 200, description = "Invitation sent successfully", body = ApiSuccessResponseInvitationReadDto),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 403, description = "Forbidden - User is not admin of the group"),
        (status = 404, description = "User or group chat not found"),
        (status = 409, description = "Invitation already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Invitation",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn send(
    Extension(current_user): Extension<User>,
    State(state): State<InvitationState>,
    ValidatedRequest(payload): ValidatedRequest<InvitationCreateDto>,
) -> Result<Json<ApiSuccessResponse<InvitationReadDto>>, ApiError> {
    let invitation = state.invitation_service.send(payload, current_user.id).await?;
    Ok(Json(ApiSuccessResponse::send(invitation)))
}
