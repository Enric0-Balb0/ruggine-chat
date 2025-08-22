use axum::{Extension, Json};
use axum::extract::{Path, State};
use crate::dto::user_dto::{ProfileUpdateDto, UserReadDto};
use crate::entity::user::User;
use crate::error::api_error::ApiError;
use crate::error::request_error::ValidatedRequest;
use crate::response::api_response::ApiSuccessResponse;
use crate::state::user_state::UserState;

#[utoipa::path(
    get,
    path = "/api/user/{id}",
    params(
        ("id" = i32, Path, description = "User ID to retrieve")
    ),
    responses(
        (status = 200, description = "User retrieved successfully", body = ApiSuccessResponseUserReadDto),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "User not found or invitation not belonging to the authenticated user"),
        (status = 500, description = "Internal server error")
    ),
    tag = "User",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn find_by_id(
    Extension(current_user): Extension<User>,
    State(state): State<UserState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiSuccessResponse<UserReadDto>>, ApiError> {
    let user = state.user_service.find_by_id(id).await?;

    Ok(Json(ApiSuccessResponse::send(user)))
}