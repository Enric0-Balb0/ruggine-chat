use axum::{
    routing::patch,
    Router,
};

pub fn routes() -> Router<crate::state::text_message_state::TextMessageState> {
    Router::new()
        .route("/update_read_at", patch(crate::handler::text_message_handler::update_read_at::update_read_at))
}