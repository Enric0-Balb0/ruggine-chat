use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use crate::error::connection_error::ConnectionError;
use crate::websocket::message::WebSocketMessage;
use crate::websocket::WebSocketConnection;

#[async_trait]
#[automock]
pub trait WebSocketManagerTrait: Send + Sync {
    /// Send message to a specific connection
    async fn send_to_connection(&self, connection_id: &str, message: WebSocketMessage) -> Result<(), ConnectionError>;
    
    /// Send message to all connections of a specific user
    async fn send_to_user(&self, user_id: i32, message: WebSocketMessage) -> usize;
    
    /// Broadcast message to all connections
    async fn broadcast(&self, message: WebSocketMessage) -> usize;

    /// Register a connection
    async fn register_connection(&self, user_id: i32, sender: tokio::sync::mpsc::UnboundedSender<WebSocketMessage>,) -> Arc<WebSocketConnection>;

    /// Remove a connection
    async fn remove_connection(&self, connection_id: &str, user_id: i32);

    /// Check if a connection exists
    async fn connection_exists(&self, connection_id: &str) -> bool;
    
    /// Get connection count for a user
    async fn user_connection_count(&self, user_id: i32) -> usize;
    
    /// Get total connection count
    async fn total_connections(&self) -> usize;
}
