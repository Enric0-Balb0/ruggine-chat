use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::websocket::core::connection::WebSocketConnection;
use crate::websocket::message::WebSocketMessage;

/// Manager centralizzato per tutte le connessioni WebSocket
#[derive(Debug, Clone)]
pub struct WebSocketManager {
    /// Mappa connection_id -> WebSocketConnection
    connections: Arc<DashMap<String, Arc<WebSocketConnection>>>,
    
    /// Mappa user_id -> Set di connection_ids
    user_connections: Arc<DashMap<i32, Vec<String>>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
            user_connections: Arc::new(DashMap::new()),
        }
    }

    /// Registra una nuova connessione
    pub fn register_connection(
        &self,
        user_id: i32,
        sender: mpsc::UnboundedSender<WebSocketMessage>,
    ) -> Arc<WebSocketConnection> {
        let connection_id = Uuid::new_v4().to_string();
        let connection = Arc::new(WebSocketConnection::new(user_id, connection_id.clone(), sender));

        // Aggiungi alla mappa delle connessioni
        self.connections.insert(connection_id.clone(), connection.clone());

        // Aggiungi alla mappa user -> connessioni
        self.user_connections
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(connection_id.clone());

        info!("Registered new WebSocket connection {} for user {}", connection_id, user_id);
        debug!("Total connections: {}", self.connections.len());

        connection
    }

    /// Rimuove una connessione
    pub fn unregister_connection(&self, connection_id: &String) {
        if let Some((_, connection)) = self.connections.remove(connection_id) {
            let user_id = connection.user_id;
            
            // Chiudi la connessione
            connection.close();
            
            // Rimuovi dalla mappa user -> connessioni
            if let Some(mut user_conns) = self.user_connections.get_mut(&user_id) {
                user_conns.retain(|id| id != connection_id);
                if user_conns.is_empty() {
                    drop(user_conns);
                    self.user_connections.remove(&user_id);
                }
            }

            info!("Unregistered WebSocket connection {} for user {}", connection_id, user_id);
            debug!("Total connections: {}", self.connections.len());
        }
    }

    /// Invia un messaggio a una connessione specifica
    pub async fn send_to_connection(&self, connection_id: &str, message: WebSocketMessage) -> Result<(), String> {
        if let Some(connection) = self.connections.get(connection_id) {
            connection.send_message(message).await
        } else {
            Err(format!("Connection {} not found", connection_id))
        }
    }

    /// Invia un messaggio a tutte le connessioni di un utente
    pub async fn send_to_user(&self, user_id: i32, message: WebSocketMessage) -> usize {
        let mut sent_count = 0;
        
        if let Some(connection_ids) = self.user_connections.get(&user_id) {
            for connection_id in connection_ids.iter() {
                if let Some(connection) = self.connections.get(connection_id) {
                    if connection.send_message(message.clone()).await.is_ok() {
                        sent_count += 1;
                    } else {
                        warn!("Failed to send message to connection {}", connection_id);
                    }
                }
            }
        }

        debug!("Sent message to {}/{} connections for user {}", sent_count, 
               self.user_connections.get(&user_id).map(|conns| conns.len()).unwrap_or(0), user_id);
        
        sent_count
    }

    /// Ottiene statistiche sulle connessioni
    pub fn get_stats(&self) -> ConnectionStats {
        ConnectionStats {
            total_connections: self.connections.len(),
            total_users: self.user_connections.len(),
            active_connections: self.connections
                .iter()
                .filter(|entry| entry.value().is_active())
                .count(),
        }
    }

    /// Ottiene tutte le connessioni di un utente
    pub fn get_user_connections(&self, user_id: i32) -> Vec<Arc<WebSocketConnection>> {
        if let Some(connection_ids) = self.user_connections.get(&user_id) {
            connection_ids
                .iter()
                .filter_map(|id| self.connections.get(id).map(|conn| conn.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Chiude tutte le connessioni di un utente
    pub fn close_user_connections(&self, user_id: i32) {
        if let Some(connection_ids) = self.user_connections.get(&user_id) {
            for connection_id in connection_ids.iter() {
                if let Some(connection) = self.connections.get(connection_id) {
                    connection.close();
                }
            }
        }
    }

    /// Pulisce le connessioni inattive
    pub fn cleanup_inactive_connections(&self) {
        let inactive_connections: Vec<String> = self.connections
            .iter()
            .filter(|entry| !entry.value().is_active())
            .map(|entry| entry.key().clone())
            .collect();

        for connection_id in inactive_connections {
            self.unregister_connection(&connection_id);
        }
    }

    /// Ottiene una connessione specifica per ID
    pub fn get_connection(&self, connection_id: &str) -> Option<Arc<WebSocketConnection>> {
        self.connections.get(connection_id).map(|conn| conn.clone())
    }

    /// Conta il numero di connessioni per utente
    pub fn get_user_connection_count(&self, user_id: i32) -> usize {
        self.user_connections
            .get(&user_id)
            .map(|conns| conns.len())
            .unwrap_or(0)
    }

    /// Ottiene lista di tutti gli utenti connessi
    pub fn get_connected_users(&self) -> Vec<i32> {
        self.user_connections.iter().map(|entry| *entry.key()).collect()
    }

    /// Invia un messaggio a più utenti contemporaneamente
    pub async fn send_to_users(&self, user_ids: &[i32], message: WebSocketMessage) -> usize {
        let mut total_sent = 0;
        for &user_id in user_ids {
            total_sent += self.send_to_user(user_id, message.clone()).await;
        }
        total_sent
    }

    /// Broadcast di un messaggio a tutti gli utenti connessi
    pub async fn broadcast(&self, message: WebSocketMessage) -> usize {
        let user_ids: Vec<i32> = self.get_connected_users();
        self.send_to_users(&user_ids, message).await
    }

    /// Verifica se un utente è connesso
    pub fn is_user_connected(&self, user_id: i32) -> bool {
        self.user_connections.contains_key(&user_id)
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistiche sulle connessioni WebSocket
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConnectionStats {
    pub total_connections: usize,
    pub total_users: usize,
    pub active_connections: usize,
}
