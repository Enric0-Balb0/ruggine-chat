use axum::{
    routing::get,
    Router
    ,
};

use crate::{
    handler::websocket::chat_handler::chat_websocket_handler
    ,
    state::websocket::WebSocketState,
};

/// Routes per la sottoscrizione WebSocket ai gruppi
pub fn routes() -> Router<WebSocketState> {
    Router::new()
        .route("/chat", get(chat_websocket_handler))
}
