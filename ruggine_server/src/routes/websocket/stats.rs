/*
use axum::{
    extract::State,
    routing::get,
    Router,
    Json,
    Extension,
};

use crate::{
    dto::websocket::StatsResponse,
    state::websocket::WebSocketState,
    entity::user::User,
};

/// Routes per le statistiche WebSocket
pub fn routes() -> Router<WebSocketState> {
    Router::new()
        .route("/stats", get(get_stats_handler))
}


/// Handler per ottenere le statistiche delle connessioni WebSocket
async fn get_stats_handler(
    State(state): State<WebSocketState>,
    Extension(_user): Extension<User>, // Richiede autenticazione
) -> Json<StatsResponse> {
    let stats = state.manager.get_stats();
    let connected_users = state.manager.get_connected_users();
    
    let response = StatsResponse {
        total_connections: stats.total_connections,
        total_users: stats.total_users,
        active_connections: stats.active_connections,
        connected_users,
    };
    
    Json(response)
}
*/