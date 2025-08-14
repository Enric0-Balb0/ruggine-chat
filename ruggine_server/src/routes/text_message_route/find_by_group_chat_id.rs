use axum::{
    routing::get,
    Router,
};

pub fn routes() -> Router<crate::state::text_message_state::TextMessageState> {
    Router::new()
        .route("/group/:group_id/messages", get(crate::handler::text_message_handler::find_by_group_chat_id::find_by_group_chat_id))
}
