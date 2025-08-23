use axum::{Extension, Json};
use axum::extract::{Path, State};
use crate::dto::ApiSuccessResponseUserOptionReadDto;
use crate::dto::user_dto::UserReadDto;
use crate::entity::user::User;
use crate::error::api_error::ApiError;
use crate::response::api_response::ApiSuccessResponse;
use crate::state::user_state::UserState;

#[utoipa::path(
    get,
    path = "/api/user/username/{username}",
    params(
        ("username" = String, Path, description = "Username to retrieve")
    ),
    responses(
        (status = 200, description = "User retrieved successfully", body = ApiSuccessResponseUserOptionReadDto),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "User",
    security(
        ("bearer_auth" = []))
)]
pub async fn find_by_username(
    Extension(current_user): Extension<User>,
    State(state): State<UserState>,
    Path(username): Path<String>,
) -> Result<Json<ApiSuccessResponseUserOptionReadDto>, ApiError> {
    let user_option = state.user_service.find_by_username(username).await?;

    Ok(Json(ApiSuccessResponseUserOptionReadDto { data: user_option }))
}
