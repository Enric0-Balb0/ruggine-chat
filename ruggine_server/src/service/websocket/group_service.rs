use crate::dto::group_membership_dto::GroupMembershipReadDto;
use crate::dto::text_message_dto::{TextMessageInfoSentAtDtoUpdate, TextMessageReadDto};
use crate::entity::group_membership::GroupMembership;
use crate::error::group_chat_error::GroupChatError;
use crate::error::web_socket_error::WebSocketError;
use crate::service::group_membership_service::GroupMembershipServiceTrait;
use crate::service::text_message_service::TextMessageServiceTrait;
use crate::service::websocket::websocket_group_service_trait::WebSocketGroupServiceTrait;
use crate::websocket::core::manager::WebSocketManager;
use crate::websocket::message::{ServerEvent, WebSocketMessage, WsError};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Servizio per gestire le sottoscrizioni WebSocket ai gruppi
#[derive(Clone)]
pub struct WebSocketGroupService {
    group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
    text_message_service: Arc<dyn TextMessageServiceTrait>,
    group_subscriptions: Arc<RwLock<HashMap<i32, HashSet<String>>>>, // user_id -> connection_ids
}

impl WebSocketGroupService {
    pub fn new(
        group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
        text_message_service: Arc<dyn TextMessageServiceTrait>,
    ) -> Self {
        Self {
            group_membership_service,
            text_message_service,
            group_subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl WebSocketGroupServiceTrait for WebSocketGroupService {
    /// Sottoscrive un utente al servizio WebSocket dei gruppi
    async fn subscribe(&self, user_id: i32, connection_id: &str){
        self.group_subscriptions.write().await
            .entry(user_id)
            .or_insert_with(HashSet::new)
            .insert(connection_id.to_string());
        info!("User {} subscribed via connection {}", user_id, connection_id);
    }

    /// Rimuove la sottoscrizione di un utente
    async fn unsubscribe(&self, user_id: i32, connection_id: &str) {
        let mut subscriptions = self.group_subscriptions.write().await;

        if let Some(connections) = subscriptions.get_mut(&user_id) {
            connections.remove(connection_id); // rimuove solo la connessione specifica

            if connections.is_empty() {
                subscriptions.remove(&user_id);
            }
        }

        info!("User {} unsubscribed connection {}", user_id, connection_id);
    }

    /// Pulisce tutte le sottoscrizioni relative a una connessione chiusa
    async fn cleanup_connection(&self, connection_id: &str) {
        let mut map = self.group_subscriptions.write().await;
        for (_, connections) in map.iter_mut() {
            connections.remove(connection_id);
        }
        // Rimuovi gli utenti che non hanno più connessioni attive
        map.retain(|_, connections| !connections.is_empty());
    }

    /// Invia un messaggio a tutti i membri di un gruppo
    async fn connections_to_broadcast_new_message(
        &self,
        group_id: i32,
    ) -> Result<Vec<(i32, String)>, WebSocketError> {
        let members = self
            .group_membership_service
            .find_by_group_chat_id(group_id)
            .await
            .map_err(|_| WebSocketError::GroupChatError(GroupChatError::GroupChatNotFound))?;

        let mut connection_ids = Vec::new();

        for member in members {
            // Se l’utente ha una sottoscrizione WebSocket attiva → recupero connection_id
            if let Some(connnections) = self.group_subscriptions.read().await.get(&member.user_id).cloned() {
                for conn_id in connnections {
                    connection_ids.push((member.user_id, conn_id));
                }
            }
        }

        Ok(connection_ids)
    }

    async fn get_connections_number_for_user_id(&self, user_id: i32) -> i32 {
        match self.group_subscriptions.read().await.get(&user_id) {
            Some(connections) => connections.len() as i32,
            None => 0,
        }
    }

    async fn connections_to_broadcast_new_user_joined(
        &self,
        connected_user_id: i32,
    ) -> Result<Vec<(i32, String)>, WebSocketError> {

        let response = self.group_membership_service.find_connected_users(connected_user_id).await;
        match response {
            Ok(user_ids) => {
                let mut connection_ids = Vec::new();
                for user_id in user_ids {
                    // Se l’utente ha una sottoscrizione WebSocket attiva → recupero connection_id
                    if let Some(connnections) = self.group_subscriptions.read().await.get(&user_id).cloned() {
                        for conn_id in connnections {
                            connection_ids.push((user_id, conn_id));
                        }
                    }
                }
                Ok(connection_ids)
            }
            Err(error) => {
                error!("Something went wrong getting the connected users to user_id {}: {:?}", connected_user_id, error);
                Err(WebSocketError::GroupChatError(GroupChatError::SomethingWentWrong(error.to_string())))
            }
        }
    }

    /// Statistiche sulle sottoscrizioni
    async fn get_stats(&self) -> usize {
        self.group_subscriptions.read().await.len()
    }

    async fn update_sent_at_for_a_user(&self, user_id: i32, message_id: i32) {
        let payload = TextMessageInfoSentAtDtoUpdate {
            text_message_id: message_id,
            sent_at: Utc::now(),
        };
        match self.text_message_service.update_sent_at(user_id, payload).await {
            Ok(_) => (),
            Err(e) => warn!("Failed to update sent_at for {} in the ws service: {}", user_id, e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::group_membership::{CurrentAction, MembershipStatus};
    use crate::error::api_error::ApiError;
    use crate::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;
    use crate::service::text_message_service::text_message_service_trait::MockTextMessageServiceTrait;
    use mockall::predicate::*;

    fn create_mock_group_membership_service() -> (MockGroupMembershipServiceTrait, MockTextMessageServiceTrait) {
        (MockGroupMembershipServiceTrait::new(), MockTextMessageServiceTrait::new())
    }

    fn create_test_service_with_mock(
        mock_service: MockGroupMembershipServiceTrait,
        mock_text_message_service: MockTextMessageServiceTrait,
    ) -> WebSocketGroupService {
        let group_membership_service = Arc::new(mock_service);
        let text_message_service = Arc::new(mock_text_message_service);
        WebSocketGroupService::new(group_membership_service, text_message_service)
    }

    fn create_test_group_membership_dto(user_id: i32, group_id: i32) -> GroupMembershipReadDto {
        GroupMembershipReadDto {
            id: 1,
            user_id,
            group_chat_id: group_id,
            role: crate::entity::group_membership::MemberRole::Member,
            joined_at: chrono::Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: 1,
            current_action: CurrentAction::Waiting,
        }
    }

    #[tokio::test]
    async fn test_subscribe_success() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service.0, mock_service.1);
        let user_id = 1;
        let connection_id = "conn_123";

        // Act
        let result = service.subscribe(user_id, connection_id).await;

        // Assert
        assert_eq!(service.get_stats().await, 1);
    }

    #[tokio::test]
    async fn test_subscribe_multiple_users() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Act
        let _ = service.subscribe(1, "conn_1").await;
        let _ = service.subscribe(2, "conn_2").await;
        let _ = service.subscribe(3, "conn_3").await;

        // Assert
        assert_eq!(service.get_stats().await, 3);
    }

    #[tokio::test]
    async fn test_subscribe_multiple_connections_same_user() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service.0, mock_service.1);
        let user_id = 1;

        // Act
        let _ = service.subscribe(user_id, "conn_old").await;
        let _ = service.subscribe(user_id, "conn_new").await;

        // Assert
        assert_eq!(service.get_stats().await, 1); // Still 1 user
        // Check that both connections are stored
        let subscriptions = service.group_subscriptions.read().await;
        let connections = subscriptions.get(&user_id).unwrap();
        assert_eq!(connections.len(), 2);
        assert!(connections.contains("conn_old"));
        assert!(connections.contains("conn_new"));
    }

    #[tokio::test]
    async fn test_unsubscribe_success() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service.0, mock_service.1);
        let user_id = 1;
        let connection_id = "conn_123";

        // Setup subscription first
        let _ = service.subscribe(user_id, connection_id).await;
        assert_eq!(service.get_stats().await, 1);

        // Act
        let result = service.unsubscribe(user_id, connection_id).await;

        // Assert
        assert_eq!(service.get_stats().await, 0);
    }

    #[tokio::test]
    async fn test_unsubscribe_nonexistent_user() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service.0, mock_service.1);
        let user_id = 999;
        let connection_id = "not-existing";

        // Act
        let result = service.unsubscribe(user_id, connection_id).await;

        // Assert
        assert_eq!(service.get_stats().await, 0);
    }

    #[tokio::test]
    async fn test_cleanup_connection_removes_matching_connections() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service.0, mock_service.1);
        let connection_to_remove = "conn_remove";

        // Setup multiple subscriptions
        let _ = service.subscribe(1, connection_to_remove).await;
        let _ = service.subscribe(2, "conn_keep").await;
        let _ = service.subscribe(3, connection_to_remove).await;
        
        assert_eq!(service.get_stats().await, 3);

        // Act
        service.cleanup_connection(connection_to_remove).await;

        // Assert
        assert_eq!(service.get_stats().await, 1);
        let subscriptions = service.group_subscriptions.read().await;
        assert!(!subscriptions.contains_key(&1));
        assert!(subscriptions.contains_key(&2));
        assert!(!subscriptions.contains_key(&3));
    }

    #[tokio::test]
    async fn test_cleanup_connection_partial_removal_multiple_connections() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service.0, mock_service.1);
        let user_id = 1;

        // Setup user with multiple connections
        let _ = service.subscribe(user_id, "conn_remove").await;
        let _ = service.subscribe(user_id, "conn_keep").await;
        let _ = service.subscribe(2, "conn_other").await;
        
        assert_eq!(service.get_stats().await, 2); // 2 users

        // Act - remove only one connection for user 1
        service.cleanup_connection("conn_remove").await;

        // Assert
        assert_eq!(service.get_stats().await, 2); // Still 2 users
        let subscriptions = service.group_subscriptions.read().await;
        
        // User 1 should still exist with one connection
        assert!(subscriptions.contains_key(&1));
        let user1_connections = subscriptions.get(&1).unwrap();
        assert_eq!(user1_connections.len(), 1);
        assert!(user1_connections.contains("conn_keep"));
        
        // User 2 should be unaffected
        assert!(subscriptions.contains_key(&2));
    }

    #[tokio::test]
    async fn test_cleanup_connection_no_matches() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Setup subscriptions
        let _ = service.subscribe(1, "conn_1").await;
        let _ = service.subscribe(2, "conn_2").await;
        
        assert_eq!(service.get_stats().await, 2);

        // Act - try to cleanup a connection that doesn't exist
        service.cleanup_connection("nonexistent_conn").await;

        // Assert - no subscriptions should be removed
        assert_eq!(service.get_stats().await, 2);
    }

    #[tokio::test]
    async fn test_broadcast_to_group_success() {
        // Arrange
        let mut mock_service = create_mock_group_membership_service();
        let group_id = 1;
        let user1_id = 10;
        let user2_id = 20;
        let user3_id = 30;

        // Setup mock to return group members
        let members = vec![
            create_test_group_membership_dto(user1_id, group_id),
            create_test_group_membership_dto(user2_id, group_id),
            create_test_group_membership_dto(user3_id, group_id),
        ];

        mock_service
            .0
            .expect_find_by_group_chat_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let members = vec![
                    create_test_group_membership_dto(10, 1),
                    create_test_group_membership_dto(20, 1),
                    create_test_group_membership_dto(30, 1),
                ];
                Box::pin(async move { Ok(members) })
            });

        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Subscribe some users (not all group members have active connections)
        let _ = service.subscribe(user1_id, "conn_1").await;
        let _ = service.subscribe(user3_id, "conn_3").await;
        // user2 is not subscribed

        // Act
        let result = service.connections_to_broadcast_new_message(group_id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 2);
        assert!(connection_ids.iter().any(|(_, s)| s == "conn_1"));
        assert!(connection_ids.iter().any(|(_, s)| s == "conn_3"));
    }

    #[tokio::test]
    async fn test_broadcast_to_group_multiple_connections_per_user() {
        // Arrange
        let mut mock_service = create_mock_group_membership_service();
        let group_id = 1;
        let user1_id = 10;
        let user2_id = 20;

        // Setup mock to return group members
        mock_service
            .0
            .expect_find_by_group_chat_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let members = vec![
                    create_test_group_membership_dto(10, 1),
                    create_test_group_membership_dto(20, 1),
                ];
                Box::pin(async move { Ok(members) })
            });

        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Subscribe users with multiple connections each
        let _ = service.subscribe(user1_id, "conn_1a").await;
        let _ = service.subscribe(user1_id, "conn_1b").await;
        let _ = service.subscribe(user2_id, "conn_2a").await;
        let _ = service.subscribe(user2_id, "conn_2b").await;
        let _ = service.subscribe(user2_id, "conn_2c").await;

        // Act
        let result = service.connections_to_broadcast_new_message(group_id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 5); // 2 connections for user1 + 3 for user2
        assert!(connection_ids.iter().any(|(_, s)| s == "conn_1a"));
        assert!(connection_ids.iter().any(|(_, s)| s == "conn_1b"));
        assert!(connection_ids.iter().any(|(_, s)| s == "conn_2a"));
        assert!(connection_ids.iter().any(|(_, s)| s == "conn_2b"));
        assert!(connection_ids.iter().any(|(_, s)| s == "conn_2c"));
    }

    #[tokio::test]
    async fn test_broadcast_to_group_no_active_connections() {
        // Arrange
        let mut mock_service = create_mock_group_membership_service();
        let group_id = 1;
        let user1_id = 10;
        let user2_id = 20;

        // Setup mock to return group members
        mock_service
            .0
            .expect_find_by_group_chat_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let members = vec![
                    create_test_group_membership_dto(10, 1),
                    create_test_group_membership_dto(20, 1),
                ];
                Box::pin(async move { Ok(members) })
            });

        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Don't subscribe any users

        // Act
        let result = service.connections_to_broadcast_new_message(group_id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 0);
    }

    #[tokio::test]
    async fn test_broadcast_to_group_service_error() {
        // Arrange
        let mut mock_service = create_mock_group_membership_service();
        let group_id = 999;

        // Setup mock to return error
        mock_service
            .0
            .expect_find_by_group_chat_id()
            .with(eq(group_id))
            .times(1)
            .returning(|_| {
                Box::pin(async move {
                    Err(ApiError::GroupChatError(GroupChatError::GroupChatNotFound))
                })
            });

        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Act
        let result = service.connections_to_broadcast_new_message(group_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            WebSocketError::GroupChatError(GroupChatError::GroupChatNotFound) => {
                // Expected error
            }
            _ => panic!("Expected GroupChatNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_broadcast_to_group_empty_group() {
        // Arrange
        let mut mock_service = create_mock_group_membership_service();
        let group_id = 1;

        // Setup mock to return empty group
        mock_service
            .0
            .expect_find_by_group_chat_id()
            .with(eq(group_id))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(vec![]) }));

        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Act
        let result = service.connections_to_broadcast_new_message(group_id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 0);
    }

    #[tokio::test]
    async fn test_get_stats_empty() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Act
        let stats = service.get_stats().await;

        // Assert
        assert_eq!(stats, 0);
    }

    #[tokio::test]
    async fn test_get_stats_with_subscriptions() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Setup subscriptions
        let _ = service.subscribe(1, "conn_1").await;
        let _ = service.subscribe(2, "conn_2").await;
        let _ = service.subscribe(3, "conn_3").await;

        // Act
        let stats = service.get_stats().await;

        // Assert
        assert_eq!(stats, 3);
    }

    #[tokio::test]
    async fn test_connections_to_broadcast_new_user_joined_success() {
        // Arrange
        let mut mock_service = create_mock_group_membership_service();
        let connected_user_id = 100;
        let connected_user1_id = 10;
        let connected_user2_id = 20;
        let connected_user3_id = 30;

        // Setup mock to return connected users
        mock_service
            .0
            .expect_find_connected_users()
            .with(eq(connected_user_id))
            .times(1)
            .returning(move |_| {
                let connected_users = vec![10, 20, 30];
                Box::pin(async move { Ok(connected_users) })
            });

        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Subscribe some connected users (not all have active connections)
        let _ = service.subscribe(connected_user1_id, "conn_1").await;
        let _ = service.subscribe(connected_user3_id, "conn_3").await;
        // connected_user2 is not subscribed

        // Act
        let result = service.connections_to_broadcast_new_user_joined(connected_user_id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 2);
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user1_id && conn_id == "conn_1"));
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user3_id && conn_id == "conn_3"));
    }

    #[tokio::test]
    async fn test_connections_to_broadcast_new_user_joined_multiple_connections_per_user() {
        // Arrange
        let mut mock_service = create_mock_group_membership_service();
        let connected_user_id = 100;
        let connected_user1_id = 10;
        let connected_user2_id = 20;

        // Setup mock to return connected users
        mock_service
            .0
            .expect_find_connected_users()
            .with(eq(connected_user_id))
            .times(1)
            .returning(move |_| {
                let connected_users = vec![10, 20];
                Box::pin(async move { Ok(connected_users) })
            });

        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Subscribe users with multiple connections each
        let _ = service.subscribe(connected_user1_id, "conn_1a").await;
        let _ = service.subscribe(connected_user1_id, "conn_1b").await;
        let _ = service.subscribe(connected_user2_id, "conn_2a").await;
        let _ = service.subscribe(connected_user2_id, "conn_2b").await;
        let _ = service.subscribe(connected_user2_id, "conn_2c").await;

        // Act
        let result = service.connections_to_broadcast_new_user_joined(connected_user_id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 5); // 2 connections for user1 + 3 for user2
        assert!(connection_ids.iter().any(|(_, s)| s == "conn_1a"));
        assert!(connection_ids.iter().any(|(_, s)| s == "conn_1b"));
        assert!(connection_ids.iter().any(|(_, s)| s == "conn_2a"));
        assert!(connection_ids.iter().any(|(_, s)| s == "conn_2b"));
        assert!(connection_ids.iter().any(|(_, s)| s == "conn_2c"));
    }

    #[tokio::test]
    async fn test_connections_to_broadcast_new_user_joined_no_active_connections() {
        // Arrange
        let mut mock_service = create_mock_group_membership_service();
        let connected_user_id = 100;
        let connected_user1_id = 10;
        let connected_user2_id = 20;

        // Setup mock to return connected users
        mock_service
            .0
            .expect_find_connected_users()
            .with(eq(connected_user_id))
            .times(1)
            .returning(move |_| {
                let connected_users = vec![10, 20];
                Box::pin(async move { Ok(connected_users) })
            });

        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Don't subscribe any users - no active connections

        // Act
        let result = service.connections_to_broadcast_new_user_joined(connected_user_id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 0);
    }

    #[tokio::test]
    async fn test_connections_to_broadcast_new_user_joined_service_error() {
        // Arrange
        let mut mock_service = create_mock_group_membership_service();
        let connected_user_id = 999;

        // Setup mock to return error
        mock_service
            .0
            .expect_find_connected_users()
            .with(eq(connected_user_id))
            .times(1)
            .returning(|_| {
                Box::pin(async move {
                    Err(ApiError::GroupChatError(GroupChatError::GroupChatNotFound))
                })
            });

        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Act
        let result = service.connections_to_broadcast_new_user_joined(connected_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            WebSocketError::GroupChatError(GroupChatError::SomethingWentWrong(_)) => {
                // Expected error - service error gets wrapped as SomethingWentWrong
            }
            _ => panic!("Expected SomethingWentWrong error"),
        }
    }

    #[tokio::test]
    async fn test_connections_to_broadcast_new_user_joined_no_connected_users() {
        // Arrange
        let mut mock_service = create_mock_group_membership_service();
        let connected_user_id = 100;

        // Setup mock to return empty list of connected users
        mock_service
            .0
            .expect_find_connected_users()
            .with(eq(connected_user_id))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(vec![]) }));

        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Act
        let result = service.connections_to_broadcast_new_user_joined(connected_user_id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 0);
    }

    #[tokio::test]
    async fn test_connections_to_broadcast_new_user_joined_mixed_connections() {
        // Arrange
        let mut mock_service = create_mock_group_membership_service();
        let connected_user_id = 100;
        let connected_user1_id = 10;
        let connected_user2_id = 20;
        let connected_user3_id = 30;

        // Setup mock to return connected users
        mock_service
            .0
            .expect_find_connected_users()
            .with(eq(connected_user_id))
            .times(1)
            .returning(move |_| {
                let connected_users = vec![10, 20, 30];
                Box::pin(async move { Ok(connected_users) })
            });

        let service = create_test_service_with_mock(mock_service.0, mock_service.1);

        // Subscribe users with different connection patterns
        let _ = service.subscribe(connected_user1_id, "conn_1").await; // Single connection
        let _ = service.subscribe(connected_user2_id, "conn_2a").await; // Multiple connections
        let _ = service.subscribe(connected_user2_id, "conn_2b").await;
        // connected_user3 is not subscribed - no connection

        // Act
        let result = service.connections_to_broadcast_new_user_joined(connected_user_id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 3); // 1 + 2 + 0 connections
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user1_id && conn_id == "conn_1"));
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user2_id && conn_id == "conn_2a"));
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user2_id && conn_id == "conn_2b"));
        // Should not include connected_user3 since they're not subscribed
        assert!(!connection_ids.iter().any(|(user_id, _)| *user_id == connected_user3_id));
    }
}
