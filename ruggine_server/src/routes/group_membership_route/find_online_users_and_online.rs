use axum::{routing::get, Router};
use crate::handler::group_membership_handler::find_online_users_in_group::find_online_users_in_group;
use crate::state::group_membership_state::GroupMembershipState;

pub fn routes() -> Router<GroupMembershipState> {
    Router::new()
        .route("/online-users-in-group/:group_chat_id", get(find_online_users_in_group))
}
