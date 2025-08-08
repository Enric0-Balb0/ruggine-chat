use axum::{routing::get, Router};
use crate::handler::invitation_handler::find_by_user_id::find_by_user_id;

pub fn routes() -> Router<crate::state::invitation_state::InvitationState> {
    Router::new()
        .route("/user", get(find_by_user_id))
}
