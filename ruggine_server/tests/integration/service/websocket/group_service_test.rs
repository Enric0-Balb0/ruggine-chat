use std::sync::Arc;
use ruggine_server::service::websocket::group_service::WebSocketGroupService;
use ruggine_server::service::group_membership_service::{GroupMembershipService, GroupMembershipServiceTrait};
use ruggine_server::websocket::core::manager::WebSocketManager;
use ruggine_server::error::web_socket_error::WebSocketError;
use ruggine_server::error::group_chat_error::GroupChatError;
use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::factory::group_chat_factory::GroupChatFactory;
use ruggine_server::entity::group_membership::MemberRole;

#[cfg(test)]
mod websocket_group_service_integration_tests {
    use ruggine_server::{service::websocket::WebSocketGroupServiceTrait, utils::service_initializer::ServiceInitializer};

    use super::*;
    use crate::{cleanup_group_chat, cleanup_test_user_from_a_group_chat, cleanup_test_users, common::{
        add_test_user_to_a_group, create_test_group_chat_with_invitation_and_membership, create_test_user, get_database
    }};

    async fn create_test_service() -> Arc<dyn WebSocketGroupServiceTrait> {
        let service_initializer = ServiceInitializer::new(&get_database().await);
        service_initializer.websocket_group_service()
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_subscribe_and_unsubscribe_integration() {
        // Arrange
        let service = create_test_service().await;
        let user_id = 12345;
        let connection_id = "integration_conn_1";

        // Act & Assert - Subscribe
        let subscribe_result = service.subscribe(user_id, connection_id).await;
        assert!(subscribe_result.is_ok());
        assert_eq!(service.get_stats().await, 1);

        // Act & Assert - Unsubscribe
        let unsubscribe_result = service.unsubscribe(user_id).await;
        assert!(unsubscribe_result.is_ok());
        assert_eq!(service.get_stats().await, 0);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_cleanup_connection_integration() {
        // Arrange
        let service = create_test_service().await;
        let connection_id = "integration_cleanup_conn";

        // Setup multiple subscriptions with the same connection
        let _ = service.subscribe(1001, connection_id).await;
        let _ = service.subscribe(1002, "other_conn").await;
        let _ = service.subscribe(1003, connection_id).await;

        assert_eq!(service.get_stats().await, 3);

        // Act
        service.cleanup_connection(connection_id).await;

        // Assert
        assert_eq!(service.get_stats().await, 1);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_broadcast_to_group_with_real_data() {
        // Arrange
        let service = create_test_service().await;
        
        // Create test data using factories and common utilities
        let (admin_user, _) = create_test_user("ws_group_admin").await;
        let (member1, _) = create_test_user("ws_group_member1").await;
        let (member2, _) = create_test_user("ws_group_member2").await;
        let (member3, _) = create_test_user("ws_group_member3").await;

        let group_chat = create_test_group_chat_with_invitation_and_membership("ws_test_group", admin_user.id).await;

        // Add members to the group
        let _membership1 = add_test_user_to_a_group(member1.id, &group_chat).await;
        let _membership2 = add_test_user_to_a_group(member2.id, &group_chat).await;
        let _membership3 = add_test_user_to_a_group(member3.id, &group_chat).await;

        // Subscribe some members to WebSocket
        let _ = service.subscribe(admin_user.id, "admin_conn").await;
        let _ = service.subscribe(member1.id, "member1_conn").await;
        let _ = service.subscribe(member3.id, "member3_conn").await;
        // member2 is not subscribed

        // Act
        let result = service.broadcast_to_group(group_chat.id).await;

        // Assert
        assert!(result.is_ok(), "Failed to broadcast to group: {:?}", result);
        let connection_ids = result.unwrap();
        
        // Should return connections for admin, member1, and member3 (3 connections)
        assert_eq!(connection_ids.len(), 3);
        assert!(connection_ids.contains(&"admin_conn".to_string()));
        assert!(connection_ids.contains(&"member1_conn".to_string()));
        assert!(connection_ids.contains(&"member3_conn".to_string()));

        // Cleanup
        cleanup_test_user_from_a_group_chat(member1.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(member2.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(member3.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_test_users(vec![admin_user.id, member1.id, member2.id, member3.id]).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_broadcast_to_nonexistent_group() {
        // Arrange
        let service = create_test_service().await;
        let nonexistent_group_id = 999999;

        // Act
        let result = service.broadcast_to_group(nonexistent_group_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            WebSocketError::GroupChatError(GroupChatError::GroupChatNotFound) => {
                // Expected error
            }
            other => panic!("Expected GroupChatNotFound error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_broadcast_to_empty_group() {
        // Arrange
        let service = create_test_service().await;
        
        // Create an empty group
        let (admin_user, _) = create_test_user("ws_empty_group_admin").await;
        let empty_group = create_test_group_chat_with_invitation_and_membership("ws_empty_group", admin_user.id).await;

        // Act
        let result = service.broadcast_to_group(empty_group.id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        
        // Should return only the admin connection (if subscribed), but admin is not in memberships
        // since the group creation doesn't automatically add the creator as a member in this websocket system
        assert_eq!(connection_ids.len(), 0);

        // Clean up
        cleanup_test_user_from_a_group_chat(admin_user.id, empty_group.id).await;
        cleanup_group_chat(empty_group.id).await;
        cleanup_test_users(vec![admin_user.id]).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_multiple_users_same_group_scenario() {
        // Arrange
        let service = create_test_service().await;
        
        // Create test scenario with multiple users in the same group
        let (admin_user, _) = create_test_user("ws_multi_admin").await;
        let (member1, _) = create_test_user("ws_multi_member1").await;
        let (member2, _) = create_test_user("ws_multi_member2").await;

        let group_chat = create_test_group_chat_with_invitation_and_membership("ws_multi_group", admin_user.id).await;

        // Add members
        let _membership1 = add_test_user_to_a_group(member1.id, &group_chat).await;
        let _membership2 = add_test_user_to_a_group(member2.id, &group_chat).await;

        // Test subscription and stats
        assert_eq!(service.get_stats().await, 0);

        let _ = service.subscribe(admin_user.id, "multi_admin_conn").await;
        assert_eq!(service.get_stats().await, 1);

        let _ = service.subscribe(member1.id, "multi_member1_conn").await;
        assert_eq!(service.get_stats().await, 2);

        let _ = service.subscribe(member2.id, "multi_member2_conn").await;
        assert_eq!(service.get_stats().await, 3);

        // Test broadcast
        let result = service.broadcast_to_group(group_chat.id).await;
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 3);

        // Test cleanup of one connection
        service.cleanup_connection("multi_member1_conn").await;
        assert_eq!(service.get_stats().await, 2);

        // Test broadcast again
        let result = service.broadcast_to_group(group_chat.id).await;
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 2); // Only member2 now and admin

        // Test unsubscribe all
        let _ = service.unsubscribe(admin_user.id).await;
        let _ = service.unsubscribe(member2.id).await;
        assert_eq!(service.get_stats().await, 0);

        // Test broadcast to group with no active connections
        let result = service.broadcast_to_group(group_chat.id).await;
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 0);

        // Clean up
        cleanup_test_user_from_a_group_chat(member1.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(member2.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_test_users(vec![admin_user.id, member1.id, member2.id]).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_multiple_connections_per_user() {
        // Arrange
        let service = create_test_service().await;
        let user_id = 54321;

        // Act - Subscribe with first connection
        let _ = service.subscribe(user_id, "first_connection").await;
        assert_eq!(service.get_stats().await, 1);

        // Act - Subscribe with second connection (should add, not overwrite)
        let _ = service.subscribe(user_id, "second_connection").await;
        assert_eq!(service.get_stats().await, 1); // Still one user but with multiple connections

        // Act - Subscribe with third connection for same user
        let _ = service.subscribe(user_id, "third_connection").await;
        assert_eq!(service.get_stats().await, 1); // Still one user with multiple connections
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_broadcast_with_multiple_connections_per_user() {
        // Arrange
        let service = create_test_service().await;
        
        let (admin_user, _) = create_test_user("ws_multi_conn_admin").await;
        let (member1, _) = create_test_user("ws_multi_conn_member1").await;

        let group_chat = create_test_group_chat_with_invitation_and_membership("ws_multi_conn_group", admin_user.id).await;
        let _membership1 = add_test_user_to_a_group(member1.id, &group_chat).await;

        // Subscribe admin with multiple connections
        let _ = service.subscribe(admin_user.id, "admin_conn_1").await;
        let _ = service.subscribe(admin_user.id, "admin_conn_2").await;
        let _ = service.subscribe(admin_user.id, "admin_conn_3").await;

        // Subscribe member with single connection
        let _ = service.subscribe(member1.id, "member1_conn").await;

        assert_eq!(service.get_stats().await, 2); // Two users

        // Act
        let result = service.broadcast_to_group(group_chat.id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        
        // Should return 4 connections total (3 for admin + 1 for member1)
        assert_eq!(connection_ids.len(), 4);
        assert!(connection_ids.contains(&"admin_conn_1".to_string()));
        assert!(connection_ids.contains(&"admin_conn_2".to_string()));
        assert!(connection_ids.contains(&"admin_conn_3".to_string()));
        assert!(connection_ids.contains(&"member1_conn".to_string()));

        // Test cleanup of one connection for admin
        service.cleanup_connection("admin_conn_2").await;
        
        let result = service.broadcast_to_group(group_chat.id).await;
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 3); // Should have 3 connections left

        // Cleanup
        cleanup_test_user_from_a_group_chat(member1.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_test_users(vec![admin_user.id, member1.id]).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_concurrent_operations() {
        // Arrange
        let service = Arc::new(create_test_service().await);
        let mut handles = vec![];

        // Act - Perform concurrent subscribe operations
        for i in 0..10 {
            let service_clone = Arc::clone(&service);
            let handle = tokio::spawn(async move {
                let user_id = 2000 + i;
                let connection_id = format!("concurrent_conn_{}", i);
                service_clone.subscribe(user_id, &connection_id).await
            });
            handles.push(handle);
        }

        // Wait for all subscriptions to complete
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        // Assert
        assert_eq!(service.get_stats().await, 10);

        // Test concurrent cleanup
        let mut cleanup_handles = vec![];
        for i in 0..5 {
            let service_clone = Arc::clone(&service);
            let handle = tokio::spawn(async move {
                let connection_id = format!("concurrent_conn_{}", i);
                service_clone.cleanup_connection(&connection_id).await;
            });
            cleanup_handles.push(handle);
        }

        for handle in cleanup_handles {
            handle.await.unwrap();
        }

        // Should have 5 subscriptions left
        assert_eq!(service.get_stats().await, 5);
    }
}
