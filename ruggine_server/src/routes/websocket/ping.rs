use axum::{
    extract::Path,
    routing::post,
    Router,
    Json,
    Extension,
    extract::State,
};

use crate::{
    dto::websocket::PingResponse,
    state::websocket::WebSocketState,
    entity::user::User,
    handler::websocket::ping_handler::ping_user_handler,
};

/// Routes per il ping degli utenti WebSocket
pub fn routes() -> Router<WebSocketState> {
    Router::new()
        .route("/ping/:user_id", post(ping_user_handler))
}
