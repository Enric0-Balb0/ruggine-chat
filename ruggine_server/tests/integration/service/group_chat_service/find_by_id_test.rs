use std::sync::Arc;
use ruggine_server::service::group_chat_service::{GroupChatService, GroupChatServiceTrait};
use ruggine_server::repository::group_chat_repository::{GroupChatRepositoryTrait};
use ruggine_server::service::user_service::{UserServiceTrait};
use ruggine_server::factory::group_chat_factory::GroupChatFactory;
use ruggine_server::error::api_error::ApiError;
use crate::common::{get_database, create_test_user, cleanup_user_by_email, cleanup_group_chat};

#[cfg(test)]
mod group_chat_service_find_by_id_integration_tests {
    use ruggine_server::error::group_chat_error::GroupChatError;
    use ruggine_server::utils::service_initializer::ServiceInitializer;
    use crate::{cleanup_test_user_from_a_group_chat, cleanup_group_membership, cleanup_invitation};
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_existing_group() {
        // Arrange
        let (user, _) = create_test_user("service_find_creator").await;
        let db = get_database().await;

        let service_init = ServiceInitializer::new(&db);
        let group_chat_service = service_init.group_chat_service();
        let group_membership_service = service_init.group_membership_service();

        // First create a group to find
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("service_find_test");
        let created_group = group_chat_service.create(create_dto.clone(), user.id).await.unwrap();

        // Act
        let result = group_chat_service.find_by_id(created_group.id).await;

        // Assert
        assert!(result.is_ok(), "Should find the group successfully");
        let found_group = result.unwrap();
        
        assert_eq!(found_group.id, created_group.id);
        assert_eq!(found_group.name, create_dto.name);
        assert_eq!(found_group.description, create_dto.description);
        assert_eq!(found_group.created_by, user.id);
        assert!(found_group.created_at <= found_group.updated_at);

        // Cleanup
        cleanup_test_user_from_a_group_chat(user.id, found_group.id).await;
        cleanup_group_chat(created_group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_non_existent_group() {
        // Arrange
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let group_chat_service = service_init.group_chat_service();
        let non_existent_id = 99999;

        // Act
        let result = group_chat_service.find_by_id(non_existent_id).await;

        // Assert
        assert!(result.is_err(), "Should not find non-existent group");
        match result.unwrap_err() {
            ApiError::GroupChatError(GroupChatError::GroupChatNotFound) => {
                // This is expected for row not found
            },
            other => panic!("Expected SomethingWentWrong error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_invalid_id() {
        // Arrange
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let group_chat_service = service_init.group_chat_service();
        let invalid_id = -1;

        // Act
        let result = group_chat_service.find_by_id(invalid_id).await;

        // Assert
        assert!(result.is_err(), "Should not find group with invalid ID");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_multiple_groups() {
        // Arrange
        let (user, _) = create_test_user("multi_service_find_creator").await;
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let group_chat_service = service_init.group_chat_service();
        let group_membership_service = service_init.group_membership_service();

        // Create multiple groups
        let create_dto1 = GroupChatFactory::unique_fake_group_chat_create_dto("multi_service_1");
        let create_dto2 = GroupChatFactory::unique_fake_group_chat_create_dto("multi_service_2");

        let group1 = group_chat_service.create(create_dto1.clone(), user.id).await.unwrap();
        let group2 = group_chat_service.create(create_dto2.clone(), user.id).await.unwrap();

        // Act
        let result1 = group_chat_service.find_by_id(group1.id).await;
        let result2 = group_chat_service.find_by_id(group2.id).await;

        // Assert
        assert!(result1.is_ok(), "Should find first group");
        assert!(result2.is_ok(), "Should find second group");

        let found_group1 = result1.unwrap();
        let found_group2 = result2.unwrap();

        assert_eq!(found_group1.id, group1.id);
        assert_eq!(found_group2.id, group2.id);
        assert_eq!(found_group1.name, create_dto1.name);
        assert_eq!(found_group2.name, create_dto2.name);
        assert_ne!(found_group1.id, found_group2.id);

        // Cleanup
        cleanup_test_user_from_a_group_chat(user.id, found_group1.id).await;
        cleanup_test_user_from_a_group_chat(user.id, found_group2.id).await;
        cleanup_group_chat(group1.id).await;
        cleanup_group_chat(group2.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_after_create() {
        // Arrange
        let (user, _) = create_test_user("create_find_service_creator").await;
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let group_chat_service = service_init.group_chat_service();
        let group_membership_service = service_init.group_membership_service();

        // Use factory with specific data
        let create_dto = GroupChatFactory::with_name_dto(
            GroupChatFactory::with_description_dto(
                GroupChatFactory::fake_group_chat_create_dto(),
                "Service test description".to_string()
            ),
            "Service Test Group".to_string()
        );

        // Act
        let created_group = group_chat_service.create(create_dto.clone(), user.id).await.unwrap();
        let found_result = group_chat_service.find_by_id(created_group.id).await;

        // Assert
        assert!(found_result.is_ok(), "Should find newly created group");
        let found_group = found_result.unwrap();
        
        assert_eq!(found_group.id, created_group.id);
        assert_eq!(found_group.name, "Service Test Group");
        assert_eq!(found_group.description, "Service test description");
        assert_eq!(found_group.created_by, user.id);

        // Cleanup
        cleanup_test_user_from_a_group_chat(user.id, found_group.id).await;
        cleanup_group_chat(created_group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_concurrent_access() {
        // Arrange
        let (user, _) = create_test_user("concurrent_service_find_creator").await;
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let group_chat_service = service_init.group_chat_service();
        let group_membership_service = service_init.group_membership_service();

        // Create a group first
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("concurrent_service_find");
        let created_group = group_chat_service.create(create_dto, user.id).await.unwrap();

        let mut handles = vec![];

        // Spawn concurrent find operations
        for i in 0..5 {
            let service_clone = Arc::clone(&group_chat_service);
            
            let handle = tokio::spawn(async move {
                let result = service_clone.find_by_id(created_group.id).await;
                (i, result)
            });
            handles.push(handle);
        }

        // Wait for all finds
        let results = futures::future::join_all(handles).await;

        // Assert
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Task {i} panicked");
            let (task_id, find_result) = result.as_ref().unwrap();
            assert!(find_result.is_ok(), "Find {task_id} failed: {:?}", find_result);
            
            let found_group = find_result.as_ref().unwrap();
            assert_eq!(found_group.id, created_group.id);
        }

        // Cleanup
        cleanup_test_user_from_a_group_chat(user.id, created_group.id).await;
        cleanup_group_chat(created_group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_verify_timestamps() {
        // Arrange
        let (user, _) = create_test_user("timestamp_service_find_creator").await;
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let group_chat_service = service_init.group_chat_service();
        let group_membership_service = service_init.group_membership_service();

        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("timestamp_service_test");

        // Act
        let created_group = group_chat_service.create(create_dto, user.id).await.unwrap();
        let found_result = group_chat_service.find_by_id(created_group.id).await;

        // Assert
        assert!(found_result.is_ok(), "Should find group successfully");
        let found_group = found_result.unwrap();

        // Verify timestamps are reasonable
        assert!(found_group.created_at <= found_group.updated_at);
        assert!(found_group.created_at <= chrono::Utc::now());
        assert!(found_group.updated_at <= chrono::Utc::now());

        // Cleanup
        let group_membership = group_membership_service.find_by_user_id_and_group_id(user.id, found_group.id).await.unwrap();
        cleanup_group_membership(group_membership.id).await;
        cleanup_invitation(group_membership.invitation_id).await;
        cleanup_group_chat(created_group.id).await;
        cleanup_user_by_email(user.email).await;
    }

}
