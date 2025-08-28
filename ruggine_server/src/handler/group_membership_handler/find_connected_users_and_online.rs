use crate::error::api_error::ApiError;
use crate::state::group_membership_state::GroupMembershipState;
use crate::entity::user::User;
use axum::{extract::State, Extension, Json};
use crate::service::group_membership_service::GroupMembershipServiceTrait;
use crate::dto::ApiSuccessResponseVecUserId;


#[utoipa::path(
    get,
    path = "/api/group_membership/connected_users_online",
    responses(
        (status = 200, description = "Connected online users retrieved successfully", body = ApiSuccessResponseVecUserId),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "GroupMembership",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn find_connected_users_and_online(
    Extension(current_user): Extension<User>,
    State(state): State<GroupMembershipState>,
) -> Result<Json<ApiSuccessResponseVecUserId>, ApiError> {
    // Retrieve all connected users that are currently online for the authenticated user
    let connected_user_ids = state.group_membership_service.find_connected_users_and_online(current_user.id).await?;
    Ok(Json(ApiSuccessResponseVecUserId { data: connected_user_ids }))
}
