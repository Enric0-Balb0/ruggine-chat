use axum::{routing::get, Router};
use crate::handler::invitation_handler::find_by_id_and_user_id::find_by_id_and_user_id;
use crate::state::invitation_state::InvitationState;

pub fn routes() -> Router<InvitationState> {
    Router::new()
        .route("/:id", get(find_by_id_and_user_id))
}
