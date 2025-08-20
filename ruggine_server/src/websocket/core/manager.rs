use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::websocket::message::WebSocketMessage;
use crate::error::connection_error::ConnectionError;
use uuid::Uuid;
use tracing::{info, debug, warn};
use crate::websocket::WebSocketConnection;

#[derive(Debug, Default)]
pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<String, Arc<WebSocketConnection>>>>,
    user_connections: Arc<RwLock<HashMap<i32, HashSet<String>>>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn register_connection(self: &Arc<Self>, user_id: i32, sender: tokio::sync::mpsc::UnboundedSender<WebSocketMessage>) -> Arc<WebSocketConnection> {
        let connection_id = Uuid::new_v4().to_string();
        let connection = WebSocketConnection::new(user_id, connection_id.clone(), sender, None, Arc::downgrade(self));

        self.connections.write().await.insert(connection_id.clone(), connection.clone());
        self.user_connections.write().await.entry(user_id).or_default().insert(connection_id.clone());

        info!("Registered connection {} for user {}", connection_id, user_id);
        connection
    }

    pub async fn remove_connection(&self, connection_id: &str, user_id: i32) {
        {
            let mut connections = self.connections.write().await;
            connections.remove(connection_id);
        }

        {
            let mut user_connections = self.user_connections.write().await;
            if let Some(conns) = user_connections.get_mut(&user_id) {
                conns.remove(connection_id);

                // se non ha più connessioni attive → rimuoviamo l’entry
                if conns.is_empty() {
                    user_connections.remove(&user_id);
                }
            }
        }

        info!("Removed connection {} for user {}", connection_id, user_id);
    }

    pub async fn send_to_connection(&self, connection_id: &str, message: WebSocketMessage) -> Result<(), ConnectionError> {
        let conns = self.connections.read().await;
        if let Some(conn) = conns.get(connection_id) {
            conn.send_message(message).await
        } else {
            Err(ConnectionError::ConnectionNotFound)
        }
    }

    pub async fn send_to_user(&self, user_id: i32, message: WebSocketMessage) -> usize {
        let conns = self.connections.read().await;
        let user_map = self.user_connections.read().await;

        let mut sent = 0;
        if let Some(connection_ids) = user_map.get(&user_id) {
            for id in connection_ids {
                if let Some(conn) = conns.get(id) {
                    if conn.send_message(message.clone()).await.is_ok() {
                        sent += 1;
                    } else {
                        warn!("Failed to send to connection {}", id);
                    }
                }
            }
        }
        sent
    }

    pub async fn broadcast(&self, message: WebSocketMessage) -> usize {
        let user_ids: Vec<i32> = self.user_connections.read().await.keys().copied().collect();
        let mut total = 0;
        for user_id in user_ids {
            total += self.send_to_user(user_id, message.clone()).await;
        }
        total
    }

}
