pub mod text_message_dto;

use serde::{Deserialize, Serialize};

/// Parametri della query per la connessione WebSocket
#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    pub token: Option<String>,
}

/// Struttura per la risposta del ping
#[derive(Debug, Serialize)]
pub struct PingResponse {
    pub success: bool,
    pub message: String,
    pub connections_pinged: usize,
}

/// Struttura per la risposta delle statistiche
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_connections: usize,
    pub total_users: usize,
    pub active_connections: usize,
    pub connected_users: Vec<i32>,
}