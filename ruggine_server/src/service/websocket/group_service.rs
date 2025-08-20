use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use crate::dto::group_membership_dto::GroupMembershipReadDto;
use crate::dto::websocket::text_message_dto::NewMessage;
use crate::entity::group_membership::GroupMembership;
use crate::error::group_chat_error::GroupChatError;
use crate::error::web_socket_error::WebSocketError;
use crate::service::group_membership_service::GroupMembershipServiceTrait;
use crate::websocket::core::manager::WebSocketManager;
use crate::websocket::message::{WebSocketMessage, ServerEvent, WsError};

/// Servizio per gestire le sottoscrizioni WebSocket ai gruppi
#[derive(Clone)]
pub struct WebSocketGroupService {
    pub ws_manager: Arc<WebSocketManager>,
    group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
    group_subscriptions: Arc<RwLock<HashMap<i32, String>>>, // user_id -> connection_id
}

impl WebSocketGroupService {
    pub fn new(
        ws_manager: Arc<WebSocketManager>,
        group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
    ) -> Self {
        Self {
            ws_manager,
            group_membership_service,
            group_subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Sottoscrive un utente al servizio WebSocket dei gruppi
    pub async fn subscribe(&self, user_id: i32, connection_id: &str) -> Result<(), WebSocketError> {
        self.group_subscriptions.write().await.insert(user_id, connection_id.to_string());
        info!("User {} subscribed via connection {}", user_id, connection_id);
        Ok(())
    }

    /// Rimuove la sottoscrizione di un utente
    pub async fn unsubscribe(&self, user_id: i32) -> Result<(), WebSocketError> {
        self.group_subscriptions.write().await.remove(&user_id);
        info!("User {} unsubscribed from group service", user_id);
        Ok(())
    }

    /// Pulisce tutte le sottoscrizioni relative a una connessione chiusa
    pub async fn cleanup_connection(&self, connection_id: &str) {
        let mut map = self.group_subscriptions.write().await;
        map.retain(|_, conn_id| conn_id != connection_id);
    }

    /// Invia un messaggio a tutti i membri di un gruppo
    pub async fn broadcast_to_group(
        &self,
        group_id: i32,
    ) -> Result<Vec<String>, WebSocketError> {
        let members = self
            .group_membership_service
            .find_by_group_id(group_id)
            .await
            .map_err(|_| WebSocketError::GroupChatError(GroupChatError::GroupChatNotFound))?;

        let mut connection_ids = Vec::new();

        for member in members {
            // Se l’utente ha una sottoscrizione WebSocket attiva → recupero connection_id
            if let Some(conn_id) = self.group_subscriptions.read().await.get(&member.user_id).cloned() {
                connection_ids.push(conn_id);
            }
        }

        Ok(connection_ids)
    }

    /// Statistiche sulle sottoscrizioni
    pub async fn get_stats(&self) -> usize {
        self.group_subscriptions.read().await.len()
    }

    /*
    /// Opzionale: invia eventi tipo GroupJoined / GroupLeft
     pub async fn send_group_event(&self, user_id: i32, event: ServerEvent) {
        if let Some(conn_id) = self.group_subscriptions.read().await.get(&user_id).cloned() {
            if let Some(conn) = self.ws_manager.get_connection(&conn_id) {
                let msg = WebSocketMessage::Event {
                    event,
                    timestamp: Utc::now(),
                };
                if let Err(e) = conn.send_message(msg).await {
                    warn!("Failed to send group event to {}: {}", user_id, e);
                }
            }
        }
    }*/
}
