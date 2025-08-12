use axum::{routing::patch, Router};
use crate::handler::group_membership_handler::leave_group::leave_group;
use crate::state::group_membership_state::GroupMembershipState;

pub fn routes() -> Router<GroupMembershipState> {
    Router::new()
        .route("/leave", patch(leave_group))
}
