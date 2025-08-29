use crate::error::web_socket_error::WebSocketError;
use async_trait::async_trait;
use mockall::automock;

#[async_trait]
#[automock]
pub trait WebSocketGroupServiceTrait: Send + Sync {
    /// Sottoscrive un utente al servizio WebSocket dei gruppi
    async fn subscribe(&self, user_id: i32, connection_id: &str);
    
    /// Rimuove la sottoscrizione di un utente
    async fn unsubscribe(&self, user_id: i32, connection_id: &str);
    
    /// Pulisce tutte le sottoscrizioni relative a una connessione chiusa
    async fn cleanup_connection(&self, connection_id: &str);
    
    /// Invia un messaggio a tutti i membri di un gruppo
    async fn connections_to_broadcast_new_message(&self, group_id: i32) -> Result<Vec<(i32, String)>, WebSocketError>;

    async fn connections_to_broadcast_new_user_joined(
        &self,
        connected_user_id: i32,
    ) -> Result<Vec<(i32, String)>, WebSocketError>;

    async fn get_connections_number_for_user_id(&self, user_id: i32) -> i32;
    
    /// Statistiche sulle sottoscrizioni
    async fn get_stats(&self) -> usize;

    async fn update_sent_at_for_a_user(&self, user_id: i32, message_id: i32);
}
