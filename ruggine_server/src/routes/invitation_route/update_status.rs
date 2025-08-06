use axum::{routing::patch, Router};
use crate::handler::invitation_handler::update_status::update_status;
use crate::state::invitation_state::InvitationState;

pub fn routes() -> Router<InvitationState> {
    Router::new()
        .route("/update-status", patch(update_status))
}
