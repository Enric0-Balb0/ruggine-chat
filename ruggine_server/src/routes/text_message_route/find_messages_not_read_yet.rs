use axum::{
    routing::get,
    Router,
};

pub fn routes() -> Router<crate::state::text_message_state::TextMessageState> {
    Router::new()
        .route("/group/:group_id/messages/not-read-yet", get(crate::handler::text_message_handler::find_messages_not_read_yet::find_messages_not_read_yet))
}
