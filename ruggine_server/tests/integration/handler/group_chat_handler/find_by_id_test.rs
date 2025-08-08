use ruggine_server::handler::group_chat_handler::find_by_id::find_by_id;
use ruggine_server::factory::group_chat_factory::GroupChatFactory;
use ruggine_server::service::group_chat_service::{GroupChatService, GroupChatServiceTrait};
use ruggine_server::service::user_service::{UserServiceTrait};
use ruggine_server::repository::group_chat_repository::{GroupChatRepositoryTrait};
use ruggine_server::repository::user_repository::{UserRepositoryTrait};
use ruggine_server::config::database::DatabaseTrait;
use ruggine_server::state::group_chat_state::GroupChatState;
use axum::{Extension, extract::{Path, State}};
use crate::common::{cleanup_user, cleanup_group_chat, create_test_group_chat, create_test_user};
use crate::get_database;
use std::sync::Arc;

#[cfg(test)]
mod find_by_id_handler_integration_tests {
    use ruggine_server::service::user_service::UserService;
    use ruggine_server::utils::service_initializer::ServiceInitializer;
    use crate::{clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id, create_group_chat_state};
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_existing_group() {
        // Arrange: Create user and group chat
        let (user, _) = create_test_user("find_handler_existing").await;
        let group = create_test_group_chat("find_handler_existing", user.id).await;
        
        let db = get_database().await;
        let group_chat_service = GroupChatService::new(&db);
        let user_service = UserService::new(&db);
        
        let state = GroupChatState {
            group_chat_service: Arc::new(group_chat_service),
            user_service: Arc::new(user_service),
        };

        // Act
        let result = find_by_id(
            Extension(user.clone()),
            State(state),
            Path(group.id),
        ).await;

        // Assert
        assert!(result.is_ok(), "Handler should successfully find existing group");
        let response = result.unwrap().0;
        let data = response.data();
        
        assert_eq!(data.id, group.id);
        assert_eq!(data.name, group.name);
        assert_eq!(data.description, group.description);
        assert_eq!(data.created_by, group.created_by);

        // Cleanup
        cleanup_group_chat(group.id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_non_existent_group() {
        // Arrange: Create user but no group chat
        let (user, _) = create_test_user("find_handler_non_existent").await;
        let non_existent_id = 99999;
        
        let db = get_database().await;
        let group_chat_service = GroupChatService::new(&db);
        let user_service = UserService::new(&db);
        
        let state = GroupChatState {
            group_chat_service: Arc::new(group_chat_service),
            user_service: Arc::new(user_service),
        };

        // Act
        let result = find_by_id(
            Extension(user.clone()),
            State(state),
            Path(non_existent_id),
        ).await;

        // Assert
        assert!(result.is_err(), "Handler should return error for non-existent group");

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_multiple_groups() {
        // Arrange: Create user and multiple group chats
        let (user, _) = create_test_user("find_handler_multiple").await;
        let group1 = create_test_group_chat("find_handler_multiple_1", user.id).await;
        let group2 = create_test_group_chat("find_handler_multiple_2", user.id).await;
        
        let db = get_database().await;
        let group_chat_service = GroupChatService::new(&db);
        let user_service = UserService::new(&db);
        
        let state = GroupChatState {
            group_chat_service: Arc::new(group_chat_service),
            user_service: Arc::new(user_service),
        };

        // Act: Find both groups
        let result1 = find_by_id(
            Extension(user.clone()),
            State(state.clone()),
            Path(group1.id),
        ).await;

        let result2 = find_by_id(
            Extension(user.clone()),
            State(state),
            Path(group2.id),
        ).await;

        // Assert
        assert!(result1.is_ok(), "Should find first group");
        assert!(result2.is_ok(), "Should find second group");
        
        let response1 = result1.unwrap().0;
        let response2 = result2.unwrap().0;
        
        assert_eq!(response1.data().id, group1.id);
        assert_eq!(response2.data().id, group2.id);
        assert_ne!(response1.data().name, response2.data().name);

        // Cleanup
        cleanup_group_chat(group1.id).await;
        cleanup_group_chat(group2.id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_different_users_same_group() {
        // Arrange: Create two users and one group chat
        let (user1, _) = create_test_user("find_handler_user1").await;
        let (user2, _) = create_test_user("find_handler_user2").await;
        let group = create_test_group_chat("find_handler_shared", user1.id).await;
        
        let db = get_database().await;
        let group_chat_service = GroupChatService::new(&db);
        let user_service = UserService::new(&db);
        
        let state = GroupChatState {
            group_chat_service: Arc::new(group_chat_service),
            user_service: Arc::new(user_service),
        };

        // Act: Both users try to find the same group
        let result1 = find_by_id(
            Extension(user1.clone()),
            State(state.clone()),
            Path(group.id),
        ).await;

        let result2 = find_by_id(
            Extension(user2.clone()),
            State(state),
            Path(group.id),
        ).await;

        // Assert: Both should succeed
        assert!(result1.is_ok(), "First user should find the group");
        assert!(result2.is_ok(), "Second user should find the group");
        
        let response1 = result1.unwrap().0;
        let response2 = result2.unwrap().0;
        
        assert_eq!(response1.data().id, response2.data().id);
        assert_eq!(response1.data().name, response2.data().name);

        // Cleanup
        cleanup_group_chat(group.id).await;
        cleanup_user(user1.email).await;
        cleanup_user(user2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_invalid_id() {
        // Arrange: Create user
        let (user, _) = create_test_user("find_handler_invalid").await;
        let invalid_id = -1;
        
        let db = get_database().await;
        let group_chat_service = GroupChatService::new(&db);
        let user_service = UserService::new(&db);
        
        let state = GroupChatState {
            group_chat_service: Arc::new(group_chat_service),
            user_service: Arc::new(user_service),
        };

        // Act
        let result = find_by_id(
            Extension(user.clone()),
            State(state),
            Path(invalid_id),
        ).await;

        // Assert
        assert!(result.is_err(), "Handler should return error for invalid ID");

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_after_creation() {
        // Arrange: Create user
        let (user, _) = create_test_user("find_handler_after_create").await;
        
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let group_chat_service = service_init.group_chat_service();
        let user_service = service_init.user_service();
        
        let state = GroupChatState {
            group_chat_service,
            user_service,
        };

        // Create group chat using service
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("find_handler_after_create");
        let created_group = state.group_chat_service.create(create_dto, user.id).await
            .expect("Failed to create group chat");

        // Act: Find the created group
        let result = find_by_id(
            Extension(user.clone()),
            State(state),
            Path(created_group.id),
        ).await;

        // Assert
        assert!(result.is_ok(), "Handler should find newly created group");
        let response = result.unwrap().0;
        let data = response.data();
        
        assert_eq!(data.id, created_group.id);
        assert_eq!(data.name, created_group.name);
        assert_eq!(data.description, created_group.description);
        assert_eq!(data.created_by, created_group.created_by);

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(user.id, created_group.id).await;
        cleanup_group_chat(created_group.id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_concurrent_access() {
        // Arrange: Create user and group chat
        let (user, _) = create_test_user("find_handler_concurrent").await;
        let group = create_test_group_chat("find_handler_concurrent", user.id).await;

        let state = create_group_chat_state().await;

        // Act: Multiple concurrent find operations
        let mut handles = vec![];
        for _i in 0..5 {
            let user_clone = user.clone();
            let state_clone = state.clone();
            let group_id = group.id;
            
            let handle = tokio::spawn(async move {
                find_by_id(
                    Extension(user_clone),
                    State(state_clone),
                    Path(group_id),
                ).await
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        let results = futures::future::join_all(handles).await;

        // Assert: All operations should succeed
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Task {} panicked", i);
            let find_result = result.as_ref().unwrap();
            assert!(find_result.is_ok(), "Find operation {} failed", i);

            let response = find_result.as_ref().unwrap();
            assert_eq!(response.0.data().id, group.id);
            assert_eq!(response.0.data().name, group.name);
        }

        // Cleanup
        cleanup_group_chat(group.id).await;
        cleanup_user(user.email).await;
    }
}
