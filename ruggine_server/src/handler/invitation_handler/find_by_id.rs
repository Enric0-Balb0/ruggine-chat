use crate::dto::invitation_dto::InvitationReadDto;
use crate::error::api_error::ApiError;
use crate::response::api_response::ApiSuccessResponse;
use crate::state::invitation_state::InvitationState;
use crate::entity::user::User;
use axum::{extract::{Path, State}, Extension, Json};

#[utoipa::path(
    get,
    path = "/api/invitation/{id}",
    params(
        ("id" = i32, Path, description = "Invitation ID to retrieve")
    ),
    responses(
        (status = 200, description = "Invitation retrieved successfully", body = ApiSuccessResponseInvitationReadDto),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Invitation not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Invitation",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn find_by_id(
    Extension(current_user): Extension<User>,
    State(state): State<InvitationState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiSuccessResponse<InvitationReadDto>>, ApiError> {
    let invitation = state.invitation_service.find_by_id(id, current_user.id).await?;
    Ok(Json(ApiSuccessResponse::send(invitation)))
}