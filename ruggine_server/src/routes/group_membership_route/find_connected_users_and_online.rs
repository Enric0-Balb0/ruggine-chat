use axum::{routing::get, Router};
use crate::handler::group_membership_handler::find_connected_users_and_online::find_connected_users_and_online;
use crate::state::group_membership_state::GroupMembershipState;

/// Routes for finding connected users that are currently online
/// 
/// This module defines the route for retrieving all user IDs of users
/// that share at least one group with the authenticated user and are currently online.
pub fn routes() -> Router<GroupMembershipState> {
    Router::new()
        .route("/connected_users_online", get(find_connected_users_and_online))
}
