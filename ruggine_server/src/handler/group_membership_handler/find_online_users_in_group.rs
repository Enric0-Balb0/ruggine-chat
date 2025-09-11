use crate::error::api_error::ApiError;
use crate::state::group_membership_state::GroupMembershipState;
use crate::entity::user::User;
use axum::{extract::State, Extension, Json};
use axum::extract::Path;
use crate::service::group_membership_service::GroupMembershipServiceTrait;
use crate::dto::ApiSuccessResponseVecUserId;


#[utoipa::path(
    get,
    path = "/api/group_membership/online-users-in-group/{group_chat_id}",
    responses(
        (status = 200, description = "Connected online users retrieved successfully", body = ApiSuccessResponseVecUserId),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 403, description = "Forbidden - User is not online"),
        (status = 404, description = "Group or membership not found or not belonging to the authenticated user"),
        (status = 500, description = "Internal server error")
    ),
    tag = "GroupMembership",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn find_online_users_in_group(
    Extension(current_user): Extension<User>,
    State(state): State<GroupMembershipState>,
    Path(group_chat_id): Path<i32>,
) -> Result<Json<ApiSuccessResponseVecUserId>, ApiError> {
    // Retrieve all connected users that are currently online for the authenticated user
    let connected_user_ids = state.group_membership_service.find_online_users_in_group(current_user.id, group_chat_id).await?;
    Ok(Json(ApiSuccessResponseVecUserId { data: connected_user_ids }))
}
