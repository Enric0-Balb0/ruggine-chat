use axum::{
    routing::post,
    Router,
};
use crate::state::text_message_state::TextMessageState;

pub fn routes() -> Router<TextMessageState> {
    Router::new()
        .route("/create", post(crate::handler::text_message_handler::create::create))
}
