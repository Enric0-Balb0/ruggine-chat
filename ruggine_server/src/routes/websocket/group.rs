use axum::{
    extract::{Path, State, WebSocketUpgrade, Query},
    response::Response,
    routing::get,
    Router,
    http::StatusCode,
};
use axum::extract::ws::WebSocket;
use std::sync::Arc;
use tracing::{info, warn, error};

use crate::{
    dto::websocket::WebSocketQuery,
    state::websocket::WebSocketState,
    websocket::core::connection::handle_group_websocket_connection,
};

/// Routes per la sottoscrizione WebSocket ai gruppi
pub fn routes() -> Router<WebSocketState> {
    Router::new()
        .route("/group/:group_id", get(group_websocket_handler))
}

/// Handler per l'upgrade WebSocket specifico per un gruppo
async fn group_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
    Path(group_id): Path<i32>,
    Query(params): Query<WebSocketQuery>,
) -> Result<Response, StatusCode> {
    // Verifica che il token sia presente
    let token = params.token.ok_or_else(|| {
        warn!("WebSocket group connection attempt without token");
        StatusCode::UNAUTHORIZED
    })?;

    // Valida il token e ottieni i claims
    let token_data = state
        .token_state
        .token_service
        .retrieve_token_claims(&token)
        .map_err(|e| {
            warn!("Invalid token in WebSocket group connection: {}", e);
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
    
    info!("WebSocket group connection attempt for user {} to group {} with valid token", 
          user_id, group_id);
    
    Ok(ws.on_upgrade(move |socket| handle_group_websocket_upgrade(
        socket, 
        user_id, 
        group_id,
        state.manager, 
        state.group_service
    )))
}

/// Gestisce l'upgrade della connessione WebSocket per un gruppo specifico
async fn handle_group_websocket_upgrade(
    socket: WebSocket,
    user_id: i32,
    group_id: i32,
    manager: Arc<crate::websocket::WebSocketManager>,
    group_service: Arc<crate::service::websocket::WebSocketGroupService>,
) {
    info!("WebSocket group connection established for user {} in group {}", user_id, group_id);
    
    // Usa il nuovo handler che gestisce anche i gruppi
    handle_group_websocket_connection(socket, user_id, group_id, manager.clone(), group_service.clone()).await;
    
    info!("WebSocket group connection closed for user {} in group {}", user_id, group_id);
}
