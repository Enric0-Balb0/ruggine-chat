use crate::dto::ApiSuccessResponseVecInvitationReadDto;
use crate::error::api_error::ApiError;
use crate::state::invitation_state::InvitationState;
use crate::entity::user::User;
use axum::{extract::State, Extension, Json};

#[utoipa::path(
    get,
    path = "/api/invitation/user",
    responses(
        (status = 200, description = "User invitations retrieved successfully", body = ApiSuccessResponseVecInvitationReadDto),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Invitation",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn find_by_user_id(
    Extension(current_user): Extension<User>,
    State(state): State<InvitationState>,
) -> Result<Json<ApiSuccessResponseVecInvitationReadDto>, ApiError> {
    let invitations = state.invitation_service.find_by_user_id(current_user.id).await?;
    Ok(Json(ApiSuccessResponseVecInvitationReadDto { data: invitations }))
}