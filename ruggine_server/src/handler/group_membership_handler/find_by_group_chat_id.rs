use crate::dto::group_membership_dto::GroupMembershipReadDto;
use crate::error::api_error::ApiError;
use crate::state::group_membership_state::GroupMembershipState;
use crate::entity::user::User;
use axum::{extract::{Path, State}, Extension, Json};
use crate::service::group_membership_service::GroupMembershipServiceTrait;
use crate::dto::ApiSuccessResponseVecGroupMembershipReadDto;

#[utoipa::path(
    get,
    path = "/api/group_membership/group_chat/{group_id}",
    params(
        ("group_id" = i32, Path, description = "Group ID to retrieve memberships for")
    ),
    responses(
        (status = 200, description = "Group memberships retrieved successfully", body = ApiSuccessResponseVecGroupMembershipReadDto),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Group or membership not found or not belonging to the authenticated user"),
        (status = 500, description = "Internal server error")
    ),
    tag = "GroupMembership",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn find_by_group_chat_id(
    Extension(current_user): Extension<User>,
    State(state): State<GroupMembershipState>,
    Path(group_id): Path<i32>,
) -> Result<Json<ApiSuccessResponseVecGroupMembershipReadDto>, ApiError> {
    let group_memberships = state.group_membership_service.find_by_group_chat_id_checked(group_id, current_user.id).await?;
    Ok(Json(ApiSuccessResponseVecGroupMembershipReadDto { data: group_memberships }))
}
