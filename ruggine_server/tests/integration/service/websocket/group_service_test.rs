use ruggine_server::error::group_chat_error::GroupChatError;
use ruggine_server::error::web_socket_error::WebSocketError;
use ruggine_server::repository::group_membership_repository::GroupMembershipRepositoryTrait;
use ruggine_server::service::group_membership_service::GroupMembershipServiceTrait;
use std::sync::Arc;

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
        assert_eq!(service.get_stats().await, 1);

        // Act & Assert - Unsubscribe
        let _unsubscribe_result = service.unsubscribe(user_id, connection_id).await;
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
        let result = service.connections_to_broadcast_by_group_id(group_chat.id).await;

        // Assert
        assert!(result.is_ok(), "Failed to broadcast to group: {:?}", result);
        let connection_ids = result.unwrap();
        
        // Should return connections for admin, member1, and member3 (3 connections)
        assert_eq!(connection_ids.len(), 3);
        assert!(connection_ids.iter().any(|(_, s)| s == "admin_conn"));
        assert!(connection_ids.iter().any(|(_, s)| s == "member1_conn"));
        assert!(connection_ids.iter().any(|(_, s)| s == "member3_conn"));

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
        let result = service.connections_to_broadcast_by_group_id(nonexistent_group_id).await;

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
        let result = service.connections_to_broadcast_by_group_id(empty_group.id).await;

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
        let result = service.connections_to_broadcast_by_group_id(group_chat.id).await;
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 3);

        // Test cleanup of one connection
        service.cleanup_connection("multi_member1_conn").await;
        assert_eq!(service.get_stats().await, 2);

        // Test broadcast again
        let result = service.connections_to_broadcast_by_group_id(group_chat.id).await;
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 2); // Only member2 now and admin

        // Test unsubscribe all
        let _ = service.unsubscribe(admin_user.id, "multi_admin_conn").await;
        let _ = service.unsubscribe(member2.id, "multi_member2_conn").await;
        assert_eq!(service.get_stats().await, 0);

        // Test broadcast to group with no active connections
        let result = service.connections_to_broadcast_by_group_id(group_chat.id).await;
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
        let result = service.connections_to_broadcast_by_group_id(group_chat.id).await;

        // Assert
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        
        // Should return 4 connections total (3 for admin + 1 for member1)
        assert_eq!(connection_ids.len(), 4);
        assert!(connection_ids.iter().any(|(_, s)| s == "admin_conn_1"));
        assert!(connection_ids.iter().any(|(_, s)| s == "admin_conn_2"));
        assert!(connection_ids.iter().any(|(_, s)| s == "admin_conn_3"));
        assert!(connection_ids.iter().any(|(_, s)| s == "member1_conn"));

        // Test cleanup of one connection for admin
        service.cleanup_connection("admin_conn_2").await;
        
        let result = service.connections_to_broadcast_by_group_id(group_chat.id).await;
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

    #[tokio_shared_rt::test(shared)]
    async fn test_connections_to_broadcast_new_user_joined_integration() {
        // Arrange: Create test scenario with users connected through groups
        let service = create_test_service().await;
        
        // Create users: target user will join a group, connected users are already in groups with target
        let (owner_user, _) = create_test_user("ws_new_user_owner").await;
        let (target_user, _) = create_test_user("ws_new_user_target").await;
        let (connected_user1, _) = create_test_user("ws_new_user_connected1").await;
        let (connected_user2, _) = create_test_user("ws_new_user_connected2").await;
        let (connected_user3, _) = create_test_user("ws_new_user_connected3").await;
        let (unrelated_user, _) = create_test_user("ws_new_user_unrelated").await;

        // Create group chats
        let group_chat1 = create_test_group_chat_with_invitation_and_membership("ws_new_user_group1", owner_user.id).await;
        let group_chat2 = create_test_group_chat_with_invitation_and_membership("ws_new_user_group2", owner_user.id).await;
        let group_chat3 = create_test_group_chat_with_invitation_and_membership("ws_new_user_group3", owner_user.id).await;

        // Add users to groups to create connections:
        // - target_user will be in group1 and group2
        // - connected_user1 will be in group1 (connected to target through group1)
        // - connected_user2 will be in both group1 and group2 (connected to target through both)
        // - connected_user3 will be in group2 (connected to target through group2)
        // - unrelated_user will be only in group3 (not connected to target)
        
        let _membership_target_1 = add_test_user_to_a_group(target_user.id, &group_chat1).await;
        let _membership_target_2 = add_test_user_to_a_group(target_user.id, &group_chat2).await;
        let _membership_connected1_1 = add_test_user_to_a_group(connected_user1.id, &group_chat1).await;
        let _membership_connected2_1 = add_test_user_to_a_group(connected_user2.id, &group_chat1).await;
        let _membership_connected2_2 = add_test_user_to_a_group(connected_user2.id, &group_chat2).await;
        let _membership_connected3_2 = add_test_user_to_a_group(connected_user3.id, &group_chat2).await;
        let _membership_unrelated_3 = add_test_user_to_a_group(unrelated_user.id, &group_chat3).await;

        // Subscribe some connected users to WebSocket (simulate active connections)
        let _ = service.subscribe(connected_user1.id, "connected1_conn").await;
        let _ = service.subscribe(connected_user2.id, "connected2_conn_a").await;
        let _ = service.subscribe(connected_user2.id, "connected2_conn_b").await; // Multiple connections
        let _ = service.subscribe(connected_user3.id, "connected3_conn").await;
        // unrelated_user is subscribed but shouldn't be in results
        let _ = service.subscribe(unrelated_user.id, "unrelated_conn").await;

        // Act: Get connections to broadcast when target_user joins
        let result = service.connections_to_broadcast_new_user_joined(target_user.id).await;

        // Assert: Should return connections for users connected to target_user through groups
        assert!(result.is_ok(), "Failed to get connections for new user joined: {:?}", result);
        let connection_ids = result.unwrap();
        
        // Should return 4 connections: 1 for connected_user1, 2 for connected_user2, 1 for connected_user3
        // Note: connected_user2 appears only once per connection they have, even though they share 2 groups with target
        assert_eq!(connection_ids.len(), 4);
        
        // Verify specific connections are included
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user1.id && conn_id == "connected1_conn"));
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user2.id && conn_id == "connected2_conn_a"));
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user2.id && conn_id == "connected2_conn_b"));
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user3.id && conn_id == "connected3_conn"));
        
        // Verify unrelated_user is not included (no shared groups)
        assert!(!connection_ids.iter().any(|(user_id, _)| *user_id == unrelated_user.id));
        
        // Verify target_user is not included (should not broadcast to self)
        assert!(!connection_ids.iter().any(|(user_id, _)| *user_id == target_user.id));

        // Cleanup: Clean up all users from their respective groups
        cleanup_test_user_from_a_group_chat(target_user.id, group_chat1.id).await;
        cleanup_test_user_from_a_group_chat(target_user.id, group_chat2.id).await;
        cleanup_test_user_from_a_group_chat(connected_user1.id, group_chat1.id).await;
        cleanup_test_user_from_a_group_chat(connected_user2.id, group_chat1.id).await;
        cleanup_test_user_from_a_group_chat(connected_user2.id, group_chat2.id).await;
        cleanup_test_user_from_a_group_chat(connected_user3.id, group_chat2.id).await;
        cleanup_test_user_from_a_group_chat(unrelated_user.id, group_chat3.id).await;
        cleanup_test_user_from_a_group_chat(owner_user.id, group_chat1.id).await;
        cleanup_test_user_from_a_group_chat(owner_user.id, group_chat2.id).await;
        cleanup_test_user_from_a_group_chat(owner_user.id, group_chat3.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_group_chat(group_chat3.id).await;
        cleanup_test_users(vec![owner_user.id, target_user.id, connected_user1.id, connected_user2.id, connected_user3.id, unrelated_user.id]).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_connections_to_broadcast_new_user_joined_no_connections() {
        // Arrange: User with connected users but no active WebSocket connections
        let service = create_test_service().await;
        
        let (owner_user, _) = create_test_user("ws_new_user_no_conn_owner").await;
        let (target_user, _) = create_test_user("ws_new_user_no_conn_target").await;
        let (connected_user1, _) = create_test_user("ws_new_user_no_conn_connected1").await;
        let (connected_user2, _) = create_test_user("ws_new_user_no_conn_connected2").await;

        // Create group and add users
        let group_chat = create_test_group_chat_with_invitation_and_membership("ws_new_user_no_conn_group", owner_user.id).await;
        let _membership_target = add_test_user_to_a_group(target_user.id, &group_chat).await;
        let _membership_connected1 = add_test_user_to_a_group(connected_user1.id, &group_chat).await;
        let _membership_connected2 = add_test_user_to_a_group(connected_user2.id, &group_chat).await;

        // Don't subscribe any users - no active WebSocket connections

        // Act
        let result = service.connections_to_broadcast_new_user_joined(target_user.id).await;

        // Assert: Should return empty list since no users have active connections
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 0);

        // Cleanup
        cleanup_test_user_from_a_group_chat(target_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(connected_user1.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(connected_user2.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(owner_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_test_users(vec![owner_user.id, target_user.id, connected_user1.id, connected_user2.id]).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_connections_to_broadcast_new_user_joined_isolated_user() {
        // Arrange: User with no group memberships (isolated user)
        let service = create_test_service().await;
        
        let (isolated_user, _) = create_test_user("ws_new_user_isolated").await;

        // Act: Try to get connections for user with no group connections
        let result = service.connections_to_broadcast_new_user_joined(isolated_user.id).await;

        // Assert: Should return empty list since user has no connections
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 0);

        // Cleanup
        cleanup_test_users(vec![isolated_user.id]).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_connections_to_broadcast_new_user_joined_nonexistent_user() {
        // Arrange
        let service = create_test_service().await;
        let nonexistent_user_id = 999999;

        // Act: Try to get connections for nonexistent user
        let result = service.connections_to_broadcast_new_user_joined(nonexistent_user_id).await;

        // Assert: Should handle gracefully and return empty list
        assert!(result.is_ok(), "Should handle nonexistent user gracefully");
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 0);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_connections_to_broadcast_new_user_joined_partial_subscriptions() {
        // Arrange: Some connected users have subscriptions, others don't
        let service = create_test_service().await;
        
        let (owner_user, _) = create_test_user("ws_new_user_partial_owner").await;
        let (target_user, _) = create_test_user("ws_new_user_partial_target").await;
        let (connected_user1, _) = create_test_user("ws_new_user_partial_connected1").await;
        let (connected_user2, _) = create_test_user("ws_new_user_partial_connected2").await;
        let (connected_user3, _) = create_test_user("ws_new_user_partial_connected3").await;

        // Create group and add users
        let group_chat = create_test_group_chat_with_invitation_and_membership("ws_new_user_partial_group", owner_user.id).await;
        let _membership_target = add_test_user_to_a_group(target_user.id, &group_chat).await;
        let _membership_connected1 = add_test_user_to_a_group(connected_user1.id, &group_chat).await;
        let _membership_connected2 = add_test_user_to_a_group(connected_user2.id, &group_chat).await;
        let _membership_connected3 = add_test_user_to_a_group(connected_user3.id, &group_chat).await;

        // Subscribe only some users
        let _ = service.subscribe(connected_user1.id, "connected1_conn").await;
        // connected_user2 is not subscribed
        let _ = service.subscribe(connected_user3.id, "connected3_conn").await;

        // Act
        let result = service.connections_to_broadcast_new_user_joined(target_user.id).await;

        // Assert: Should return only connections for subscribed users
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 2);
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user1.id && conn_id == "connected1_conn"));
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user3.id && conn_id == "connected3_conn"));
        // Should not include connected_user2 since they're not subscribed
        assert!(!connection_ids.iter().any(|(user_id, _)| *user_id == connected_user2.id));

        // Cleanup
        cleanup_test_user_from_a_group_chat(target_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(connected_user1.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(connected_user2.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(connected_user3.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(owner_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_test_users(vec![owner_user.id, target_user.id, connected_user1.id, connected_user2.id, connected_user3.id]).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_connections_to_broadcast_new_user_joined_multiple_connections_per_user() {
        // Arrange: Test with users having multiple WebSocket connections
        let service = create_test_service().await;
        
        let (owner_user, _) = create_test_user("ws_new_user_multi_conn_owner").await;
        let (target_user, _) = create_test_user("ws_new_user_multi_conn_target").await;
        let (connected_user1, _) = create_test_user("ws_new_user_multi_conn_connected1").await;
        let (connected_user2, _) = create_test_user("ws_new_user_multi_conn_connected2").await;

        // Create group and add users
        let group_chat = create_test_group_chat_with_invitation_and_membership("ws_new_user_multi_conn_group", owner_user.id).await;
        let _membership_target = add_test_user_to_a_group(target_user.id, &group_chat).await;
        let _membership_connected1 = add_test_user_to_a_group(connected_user1.id, &group_chat).await;
        let _membership_connected2 = add_test_user_to_a_group(connected_user2.id, &group_chat).await;

        // Subscribe users with multiple connections
        let _ = service.subscribe(connected_user1.id, "connected1_conn_a").await;
        let _ = service.subscribe(connected_user1.id, "connected1_conn_b").await;
        let _ = service.subscribe(connected_user1.id, "connected1_conn_c").await;
        let _ = service.subscribe(connected_user2.id, "connected2_conn_a").await;
        let _ = service.subscribe(connected_user2.id, "connected2_conn_b").await;

        // Act
        let result = service.connections_to_broadcast_new_user_joined(target_user.id).await;

        // Assert: Should return all connections for all connected users
        assert!(result.is_ok());
        let connection_ids = result.unwrap();
        assert_eq!(connection_ids.len(), 5); // 3 for connected_user1 + 2 for connected_user2
        
        // Verify all connections are included
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user1.id && conn_id == "connected1_conn_a"));
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user1.id && conn_id == "connected1_conn_b"));
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user1.id && conn_id == "connected1_conn_c"));
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user2.id && conn_id == "connected2_conn_a"));
        assert!(connection_ids.iter().any(|(user_id, conn_id)| *user_id == connected_user2.id && conn_id == "connected2_conn_b"));

        // Cleanup
        cleanup_test_user_from_a_group_chat(target_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(connected_user1.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(connected_user2.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(owner_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_test_users(vec![owner_user.id, target_user.id, connected_user1.id, connected_user2.id]).await;
    }
}
