use axum::{
    routing::get,
    Router
    ,
};

use crate::{
    handler::websocket::group_handler::group_websocket_handler
    ,
    state::websocket::WebSocketState,
};

/// Routes per la sottoscrizione WebSocket ai gruppi
pub fn routes() -> Router<WebSocketState> {
    Router::new()
        .route("/group", get(group_websocket_handler))
}
