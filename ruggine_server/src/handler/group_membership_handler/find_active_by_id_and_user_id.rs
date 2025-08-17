use crate::dto::group_membership_dto::GroupMembershipReadDto;
use crate::error::api_error::ApiError;
use crate::response::api_response::ApiSuccessResponse;
use crate::state::group_membership_state::GroupMembershipState;
use crate::entity::user::User;
use axum::{extract::{Path, State}, Extension, Json};
use crate::service::group_membership_service::GroupMembershipServiceTrait;

#[utoipa::path(
    get,
    path = "/api/group_membership/{id}",
    params(
        ("id" = i32, Path, description = "Group membership ID to retrieve")
    ),
    responses(
        (status = 200, description = "Group membership retrieved successfully", body = ApiSuccessResponseGroupMembershipReadDto),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Group membership not found or not belonging to the authenticated user"),
        (status = 500, description = "Internal server error")
    ),
    tag = "GroupMembership",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn find_active_by_id_and_user_id(
    Extension(current_user): Extension<User>,
    State(state): State<GroupMembershipState>,
    Path(id): Path<i32>,
) -> Result<Json<ApiSuccessResponse<GroupMembershipReadDto>>, ApiError> {
    let group_membership = state.group_membership_service.find_by_id_and_user_id(id, current_user.id).await?;
    Ok(Json(ApiSuccessResponse::send(group_membership)))
}
