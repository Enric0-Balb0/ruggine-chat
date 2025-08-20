use axum::{Extension, Json};
use axum::extract::{Path, State};
use tracing::info;
use crate::dto::websocket::PingResponse;
use crate::entity::user::User;
use crate::state::websocket::WebSocketState;
use crate::websocket::message::ControlMessage;
use crate::websocket::WebSocketMessage;

/// Handler per inviare un ping a un utente specifico
pub async fn ping_user_handler(
    State(state): State<WebSocketState>,
    Path(user_id): Path<i32>,
    Extension(_user): Extension<User>, // Richiede autenticazione
) -> Json<PingResponse> {
    info!("Sending ping to user {}", user_id);

    let ping_message = WebSocketMessage::Control(ControlMessage::Ping);
    let connections_pinged = state.manager.send_to_user(user_id, ping_message).await;

    let response = PingResponse {
        success: connections_pinged > 0,
        message: if connections_pinged > 0 {
            format!("Ping sent to {} connections for user {}", connections_pinged, user_id)
        } else {
            format!("User {} is not connected", user_id)
        },
        connections_pinged,
    };

    Json(response)
}