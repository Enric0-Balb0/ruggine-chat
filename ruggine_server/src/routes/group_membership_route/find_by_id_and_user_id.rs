use axum::{routing::get, Router};
use crate::handler::group_membership_handler::find_active_by_id_and_user_id::find_active_by_id_and_user_id;
use crate::state::group_membership_state::GroupMembershipState;

pub fn routes() -> Router<GroupMembershipState> {
    Router::new()
        .route("/:id", get(find_active_by_id_and_user_id))
}
