use axum::{routing::get, Router};
use crate::handler::invitation_handler::find_by_id::find_by_id;
use crate::state::invitation_state::InvitationState;

pub fn routes() -> Router<InvitationState> {
    Router::new()
        .route("/:id", get(find_by_id))
}
