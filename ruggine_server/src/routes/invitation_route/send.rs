use axum::{routing::post, Router};
use crate::handler::invitation_handler::send::send;
use crate::state::invitation_state::InvitationState;

pub fn routes() -> Router<InvitationState> {
    Router::new()
        .route("/send", post(send))
}
