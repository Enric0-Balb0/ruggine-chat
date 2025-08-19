use std::sync::Arc;
use dashmap::DashMap;
use tracing::{debug, info, warn, error};

use crate::websocket::core::manager::WebSocketManager;
use crate::websocket::message::WebSocketMessage;
use crate::service::group_membership_service::GroupMembershipServiceTrait;

/// Servizio per gestire le sottoscrizioni WebSocket ai gruppi
#[derive(Clone)]
pub struct WebSocketGroupService {
    /// Mappa group_id -> Set di connection_ids sottoscritti a quel gruppo
    group_subscriptions: Arc<DashMap<i32, Vec<String>>>,
    
    /// Mappa connection_id -> Set di group_ids a cui è sottoscritto
    connection_groups: Arc<DashMap<String, Vec<i32>>>,
    
    /// Riferimento al manager delle connessioni WebSocket
    ws_manager: Arc<WebSocketManager>,
    
    /// Servizio per verificare i permessi di membership nei gruppi
    group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
}

impl WebSocketGroupService {
    pub fn new(
        ws_manager: Arc<WebSocketManager>,
        group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
    ) -> Self {
        Self {
            group_subscriptions: Arc::new(DashMap::new()),
            connection_groups: Arc::new(DashMap::new()),
            ws_manager,
            group_membership_service,
        }
    }

    /// Sottoscrive una connessione a un gruppo (con verifica dei permessi)
    pub async fn subscribe_to_group(
        &self,
        connection_id: &str,
        user_id: i32,
        group_id: i32,
    ) -> Result<(), String> {
        // Verifica che l'utente sia membro del gruppo
        match self.group_membership_service
            .find_active_by_user_id_and_group_id(user_id, group_id)
            .await
        {
            Ok(_) => {
                // L'utente è membro del gruppo, procedi con la sottoscrizione
                self.add_subscription(connection_id, group_id);
                info!("User {} subscribed to group {} via connection {}", user_id, group_id, connection_id);
                
                // Invia conferma al client
                if let Some(connection) = self.ws_manager.get_connection(connection_id) {
                    let confirmation = WebSocketMessage::GroupJoined { group_id };
                    if let Err(e) = connection.send_message(confirmation).await {
                        warn!("Failed to send group join confirmation: {}", e);
                    }
                }
                
                Ok(())
            }
            Err(_) => {
                let error_msg = format!("User {} is not a member of group {}", user_id, group_id);
                warn!("{}", error_msg);
                
                // Invia errore al client
                if let Some(connection) = self.ws_manager.get_connection(connection_id) {
                    let error = WebSocketMessage::Error { message: error_msg.clone() };
                    if let Err(e) = connection.send_message(error).await {
                        warn!("Failed to send error message: {}", e);
                    }
                }
                
                Err(error_msg)
            }
        }
    }

    /// Rimuove la sottoscrizione di una connessione da un gruppo
    pub async fn unsubscribe_from_group(&self, connection_id: &str, group_id: i32) {
        self.remove_subscription(connection_id, group_id);
        info!("Connection {} unsubscribed from group {}", connection_id, group_id);
        
        // Invia conferma al client
        if let Some(connection) = self.ws_manager.get_connection(connection_id) {
            let confirmation = WebSocketMessage::GroupLeft { group_id };
            if let Err(e) = connection.send_message(confirmation).await {
                warn!("Failed to send group leave confirmation: {}", e);
            }
        }
    }

    /// Rimuove tutte le sottoscrizioni di una connessione (quando si disconnette)
    pub fn cleanup_connection(&self, connection_id: &str) {
        if let Some((_, group_ids)) = self.connection_groups.remove(connection_id) {
            for group_id in group_ids {
                if let Some(mut connections) = self.group_subscriptions.get_mut(&group_id) {
                    connections.retain(|id| id != connection_id);
                    if connections.is_empty() {
                        drop(connections);
                        self.group_subscriptions.remove(&group_id);
                    }
                }
            }
            debug!("Cleaned up all subscriptions for connection {}", connection_id);
        }
    }

    /// Invia un messaggio a tutti i membri sottoscritti di un gruppo
    pub async fn broadcast_to_group(&self, group_id: i32, message: WebSocketMessage) -> usize {
        let mut sent_count = 0;
        
        if let Some(connection_ids) = self.group_subscriptions.get(&group_id) {
            for connection_id in connection_ids.iter() {
                if let Some(connection) = self.ws_manager.get_connection(connection_id) {
                    match connection.send_message(message.clone()).await {
                        Ok(_) => {
                            sent_count += 1;
                            debug!("Sent message to connection {} in group {}", connection_id, group_id);
                        }
                        Err(e) => {
                            warn!("Failed to send message to connection {} in group {}: {}", 
                                  connection_id, group_id, e);
                        }
                    }
                } else {
                    warn!("Connection {} not found but still in group {} subscriptions", 
                          connection_id, group_id);
                }
            }
        }
        
        if sent_count > 0 {
            info!("Broadcasted message to {} connections in group {}", sent_count, group_id);
        }
        
        sent_count
    }

    /// Ottiene la lista dei gruppi a cui una connessione è sottoscritta
    pub fn get_connection_groups(&self, connection_id: &str) -> Vec<i32> {
        self.connection_groups
            .get(connection_id)
            .map(|groups| groups.clone())
            .unwrap_or_default()
    }

    /// Ottiene la lista delle connessioni sottoscritte a un gruppo
    pub fn get_group_connections(&self, group_id: i32) -> Vec<String> {
        self.group_subscriptions
            .get(&group_id)
            .map(|connections| connections.clone())
            .unwrap_or_default()
    }

    /// Ottiene statistiche sulle sottoscrizioni
    pub fn get_stats(&self) -> GroupSubscriptionStats {
        GroupSubscriptionStats {
            total_groups: self.group_subscriptions.len(),
            total_subscriptions: self.connection_groups.len(),
            groups_with_subscribers: self.group_subscriptions
                .iter()
                .filter(|entry| !entry.value().is_empty())
                .count(),
        }
    }

    // Metodi privati per gestire le sottoscrizioni
    fn add_subscription(&self, connection_id: &str, group_id: i32) {
        // Aggiungi alla mappa group -> connections
        self.group_subscriptions
            .entry(group_id)
            .or_insert_with(Vec::new)
            .push(connection_id.to_string());

        // Aggiungi alla mappa connection -> groups
        self.connection_groups
            .entry(connection_id.to_string())
            .or_insert_with(Vec::new)
            .push(group_id);
    }

    fn remove_subscription(&self, connection_id: &str, group_id: i32) {
        // Rimuovi dalla mappa group -> connections
        if let Some(mut connections) = self.group_subscriptions.get_mut(&group_id) {
            connections.retain(|id| id != connection_id);
            if connections.is_empty() {
                drop(connections);
                self.group_subscriptions.remove(&group_id);
            }
        }

        // Rimuovi dalla mappa connection -> groups
        if let Some(mut groups) = self.connection_groups.get_mut(connection_id) {
            groups.retain(|&id| id != group_id);
            if groups.is_empty() {
                drop(groups);
                self.connection_groups.remove(connection_id);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GroupSubscriptionStats {
    pub total_groups: usize,
    pub total_subscriptions: usize,
    pub groups_with_subscribers: usize,
}
