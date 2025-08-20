/*use axum::{
    extract::{State, WebSocketUpgrade, Query},
    response::Response,
    routing::get,
    Router,
    http::StatusCode,
};
use axum::extract::ws::WebSocket;
use std::sync::Arc;
use tracing::{info, warn};

use crate::{
    dto::websocket::WebSocketQuery, 
    state::websocket::WebSocketState,
    websocket::core::{connection::WebSocketConnection, manager::WebSocketManager}
};

/// Routes per la connessione WebSocket
pub fn routes() -> Router<WebSocketState> {
    Router::new()
        .route("/connect", get(websocket_handler))
}

/// Handler per l'upgrade WebSocket
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
    Query(params): Query<WebSocketQuery>,
) -> Result<Response, StatusCode> {
    // Verifica che il token sia presente
    let token = params.token.ok_or_else(|| {
        warn!("WebSocket connection attempt without token");
        StatusCode::UNAUTHORIZED
    })?;

    // Valida il token e ottieni i claims
    let token_data = state
        .token_state
        .token_service
        .retrieve_token_claims(&token)
        .map_err(|e| {
            warn!("Invalid token in WebSocket connection: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

    // Trova l'utente associato all'email dai claims
    let user = state
        .token_state
        .user_repo
        .find_by_email(token_data.claims.email)
        .await
        .ok_or_else(|| {
            warn!("User not found for email in token claims");
            StatusCode::UNAUTHORIZED
        })?;

    let user_id = user.id;
    
    info!("WebSocket connection attempt for user {} with valid token", user_id);
    
    Ok(ws.on_upgrade(move |socket| handle_websocket_upgrade(socket, user_id, state.manager)))
}

/// Gestisce l'upgrade della connessione WebSocket
async fn handle_websocket_upgrade(
    socket: WebSocket,
    user_id: i32,
    manager: Arc<WebSocketManager>,
) {
    info!("WebSocket connection established for user {}", user_id);
    
    handle_websocket_connection(socket, user_id, manager.clone()).await;
    
    info!("WebSocket connection closed for user {}", user_id);
}
*/