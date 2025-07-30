use crate::handler::group_chat_handler::find_by_id;
use crate::state::group_chat_state::GroupChatState;
use axum::{routing::get, Router};

pub fn routes() -> Router<GroupChatState> {
    Router::new().route("/:id", get(find_by_id::find_by_id))
}
