use crate::handler::group_chat_handler::create::create;
use crate::state::group_chat_state::GroupChatState;
use axum::{routing::{post}, Router};

pub fn routes() -> Router<GroupChatState> {
    Router::new()
        .route("/create", post(create))
}
