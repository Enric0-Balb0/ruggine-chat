use crate::error::api_error::ApiError;
use crate::state::group_membership_state::GroupMembershipState;
use crate::entity::user::User;
use axum::{extract::State, Extension, Json};
use crate::service::group_membership_service::GroupMembershipServiceTrait;
use crate::dto::ApiSuccessResponseVecGroupMembershipReadDto;


#[utoipa::path(
    get,
    path = "/api/group_membership/user",
    responses(
        (status = 200, description = "Active group memberships retrieved successfully", body = ApiSuccessResponseVecGroupMembershipReadDto),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "GroupMembership",
    security(
        ("bearer_auth" = [])
    )
)]


pub async fn find_by_user_id(
    Extension(current_user): Extension<User>,
    State(state): State<GroupMembershipState>,
) -> Result<Json<ApiSuccessResponseVecGroupMembershipReadDto>, ApiError> {
    // Retrieve all group memberships for the authenticated user
    let group_memberships = state.group_membership_service.find_by_user_id(current_user.id).await?;
    Ok(Json(ApiSuccessResponseVecGroupMembershipReadDto { data: group_memberships }))
}
