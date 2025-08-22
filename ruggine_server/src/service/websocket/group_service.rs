use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use async_trait::async_trait;
use crate::dto::group_membership_dto::GroupMembershipReadDto;
use crate::entity::group_membership::GroupMembership;
use crate::error::group_chat_error::GroupChatError;
use crate::error::web_socket_error::WebSocketError;
use crate::service::group_membership_service::GroupMembershipServiceTrait;
use crate::service::websocket::websocket_group_service_trait::WebSocketGroupServiceTrait;
use crate::websocket::core::manager::WebSocketManager;
use crate::websocket::message::{WebSocketMessage, ServerEvent, WsError};

/// Servizio per gestire le sottoscrizioni WebSocket ai gruppi
#[derive(Clone)]
pub struct WebSocketGroupService {
    group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
    group_subscriptions: Arc<RwLock<HashMap<i32, String>>>, // user_id -> connection_id
}

impl WebSocketGroupService {
    pub fn new(
        group_membership_service: Arc<dyn GroupMembershipServiceTrait>,
    ) -> Self {
        Self {
            group_membership_service,
            group_subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl WebSocketGroupServiceTrait for WebSocketGroupService {
    /// Sottoscrive un utente al servizio WebSocket dei gruppi
    async fn subscribe(&self, user_id: i32, connection_id: &str) -> Result<(), WebSocketError> {
        self.group_subscriptions.write().await.insert(user_id, connection_id.to_string());
        info!("User {} subscribed via connection {}", user_id, connection_id);
        Ok(())
    }

    /// Rimuove la sottoscrizione di un utente
    async fn unsubscribe(&self, user_id: i32) -> Result<(), WebSocketError> {
        self.group_subscriptions.write().await.remove(&user_id);
        info!("User {} unsubscribed from group service", user_id);
        Ok(())
    }

    /// Pulisce tutte le sottoscrizioni relative a una connessione chiusa
    async fn cleanup_connection(&self, connection_id: &str) {
        let mut map = self.group_subscriptions.write().await;
        map.retain(|_, conn_id| conn_id != connection_id);
    }

    /// Invia un messaggio a tutti i membri di un gruppo
    async fn broadcast_to_group(
        &self,
        group_id: i32,
    ) -> Result<Vec<String>, WebSocketError> {
        let members = self
            .group_membership_service
            .find_by_group_chat_id(group_id)
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
    async fn get_stats(&self) -> usize {
        self.group_subscriptions.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;
    use crate::service::websocket::websocket_group_service_trait::MockWebSocketGroupServiceTrait;
    use mockall::predicate::*;
    use crate::error::api_error::ApiError;
    use crate::factory::user_factory::UserFactory;
    use crate::factory::group_chat_factory::GroupChatFactory;
    use crate::entity::group_membership::MembershipStatus;

    fn create_mock_group_membership_service() -> MockGroupMembershipServiceTrait {
        MockGroupMembershipServiceTrait::new()
    }

    fn create_test_service_with_mock(
        mock_service: MockGroupMembershipServiceTrait,
    ) -> WebSocketGroupService {
        let group_membership_service = Arc::new(mock_service);
        WebSocketGroupService::new(group_membership_service)
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
        }
    }

    #[tokio::test]
    async fn test_subscribe_success() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service);
        let user_id = 1;
        let connection_id = "conn_123";

        // Act
        let result = service.subscribe(user_id, connection_id).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(service.get_stats().await, 1);
    }

    #[tokio::test]
    async fn test_subscribe_multiple_users() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service);

        // Act
        let _ = service.subscribe(1, "conn_1").await;
        let _ = service.subscribe(2, "conn_2").await;
        let _ = service.subscribe(3, "conn_3").await;

        // Assert
        assert_eq!(service.get_stats().await, 3);
    }

    #[tokio::test]
    async fn test_subscribe_overwrite_existing_user() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service);
        let user_id = 1;

        // Act
        let _ = service.subscribe(user_id, "conn_old").await;
        let _ = service.subscribe(user_id, "conn_new").await;

        // Assert
        assert_eq!(service.get_stats().await, 1);
        // Check that the connection was overwritten
        let subscriptions = service.group_subscriptions.read().await;
        assert_eq!(subscriptions.get(&user_id).unwrap(), "conn_new");
    }

    #[tokio::test]
    async fn test_unsubscribe_success() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service);
        let user_id = 1;
        let connection_id = "conn_123";

        // Setup subscription first
        let _ = service.subscribe(user_id, connection_id).await;
        assert_eq!(service.get_stats().await, 1);

        // Act
        let result = service.unsubscribe(user_id).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(service.get_stats().await, 0);
    }

    #[tokio::test]
    async fn test_unsubscribe_nonexistent_user() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service);
        let user_id = 999;

        // Act
        let result = service.unsubscribe(user_id).await;

        // Assert
        assert!(result.is_ok()); // Should not fail even if user doesn't exist
        assert_eq!(service.get_stats().await, 0);
    }

    #[tokio::test]
    async fn test_cleanup_connection_removes_matching_connections() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service);
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
    async fn test_cleanup_connection_no_matches() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service);

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

        let service = create_test_service_with_mock(mock_service);

        // Subscribe some users (not all group members have active connections)
        let _ = service.subscribe(user1_id, "conn_1").await;
        let _ = service.subscribe(user3_id, "conn_3").await;
        // user2 is not subscribed

        // Act
        let result = service.broadcast_to_group(group_id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 2);
        assert!(connection_ids.contains(&"conn_1".to_string()));
        assert!(connection_ids.contains(&"conn_3".to_string()));
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

        let service = create_test_service_with_mock(mock_service);

        // Don't subscribe any users

        // Act
        let result = service.broadcast_to_group(group_id).await;

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
            .expect_find_by_group_chat_id()
            .with(eq(group_id))
            .times(1)
            .returning(|_| {
                Box::pin(async move {
                    Err(ApiError::GroupChatError(GroupChatError::GroupChatNotFound))
                })
            });

        let service = create_test_service_with_mock(mock_service);

        // Act
        let result = service.broadcast_to_group(group_id).await;

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
            .expect_find_by_group_chat_id()
            .with(eq(group_id))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(vec![]) }));

        let service = create_test_service_with_mock(mock_service);

        // Act
        let result = service.broadcast_to_group(group_id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 0);
    }

    #[tokio::test]
    async fn test_get_stats_empty() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service);

        // Act
        let stats = service.get_stats().await;

        // Assert
        assert_eq!(stats, 0);
    }

    #[tokio::test]
    async fn test_get_stats_with_subscriptions() {
        // Arrange
        let mock_service = create_mock_group_membership_service();
        let service = create_test_service_with_mock(mock_service);

        // Setup subscriptions
        let _ = service.subscribe(1, "conn_1").await;
        let _ = service.subscribe(2, "conn_2").await;
        let _ = service.subscribe(3, "conn_3").await;

        // Act
        let stats = service.get_stats().await;

        // Assert
        assert_eq!(stats, 3);
    }
}
