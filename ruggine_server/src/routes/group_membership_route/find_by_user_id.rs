use axum::{routing::get, Router};
use crate::handler::group_membership_handler::find_by_user_id::find_by_user_id;
use crate::state::group_membership_state::GroupMembershipState;

/// Routes for finding group memberships by user ID
/// 
/// This module defines the route for retrieving all group memberships 
/// for the currently authenticated user.
pub fn routes() -> Router<GroupMembershipState> {
    Router::new()
        .route("/user", get(find_by_user_id))
}
