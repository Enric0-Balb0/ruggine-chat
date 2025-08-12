use crate::dto::group_membership_dto::{GroupMembershipReadDto, LeaveGroupMembershipDto};
use crate::error::api_error::ApiError;
use crate::response::api_response::ApiSuccessResponse;
use crate::state::group_membership_state::GroupMembershipState;
use crate::entity::user::User;
use axum::{extract::State, Extension, Json};
use crate::service::group_membership_service::GroupMembershipServiceTrait;
use crate::error::request_error::ValidatedRequest;

#[utoipa::path(
    patch,
    path = "/api/group_membership/leave",
    request_body = LeaveGroupMembershipDto,
    responses(
        (status = 200, description = "Successfully left the group", body = ApiSuccessResponseGroupMembershipReadDto),
        (status = 400, description = "Invalid input data"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Group membership not found or not belonging to the authenticated user"),
        (status = 409, description = "User already left the group"),
        (status = 500, description = "Internal server error")
    ),
    tag = "GroupMembership",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn leave_group(
    Extension(current_user): Extension<User>,
    State(state): State<GroupMembershipState>,
    ValidatedRequest(payload): ValidatedRequest<LeaveGroupMembershipDto>,
) -> Result<Json<ApiSuccessResponse<GroupMembershipReadDto>>, ApiError> {
    let group_membership = state.group_membership_service.leave_group(payload, current_user.id).await?;
    Ok(Json(ApiSuccessResponse::send(group_membership)))
}
