use crate::error::web_socket_error::WebSocketError;
use async_trait::async_trait;
use mockall::automock;

#[async_trait]
#[automock]
pub trait WebSocketGroupServiceTrait: Send + Sync {
    /// Sottoscrive un utente al servizio WebSocket dei gruppi
    async fn subscribe(&self, user_id: i32, connection_id: &str) -> Result<(), WebSocketError>;
    
    /// Rimuove la sottoscrizione di un utente
    async fn unsubscribe(&self, user_id: i32) -> Result<(), WebSocketError>;
    
    /// Pulisce tutte le sottoscrizioni relative a una connessione chiusa
    async fn cleanup_connection(&self, connection_id: &str);
    
    /// Invia un messaggio a tutti i membri di un gruppo
    async fn broadcast_to_group(&self, group_id: i32) -> Result<Vec<String>, WebSocketError>;
    
    /// Statistiche sulle sottoscrizioni
    async fn get_stats(&self) -> usize;
}
