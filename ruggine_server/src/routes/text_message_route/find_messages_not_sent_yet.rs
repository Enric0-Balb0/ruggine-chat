use axum::{
    routing::get,
    Router,
};

pub fn routes() -> Router<crate::state::text_message_state::TextMessageState> {
    Router::new()
        .route("/group/:group_id/messages/not-sent-yet", get(crate::handler::text_message_handler::find_messages_not_sent_yet::find_messages_not_sent_yet))
}
