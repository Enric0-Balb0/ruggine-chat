use axum::{routing::get, Router};
use crate::{handler::group_membership_handler::find_by_group_id::find_by_group_id, state::group_membership_state::GroupMembershipState};

pub fn routes() -> Router<GroupMembershipState> {
    Router::new()
    .route("/group/:group_id", get(find_by_group_id))
}
