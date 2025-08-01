use std::sync::Arc;
use ruggine_server::repository::group_chat_repository::{GroupChatRepository, GroupChatRepositoryTrait};
use ruggine_server::factory::group_chat_factory::GroupChatFactory;
use crate::common::{get_database, create_test_user, cleanup_user, cleanup_group_chat};

#[cfg(test)]
mod group_chat_repository_find_by_id_integration_tests {
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_success() {
        // Arrange
        let (user, _) = create_test_user("find_by_id_creator").await;
        let db = get_database().await;
        let repository = GroupChatRepository::new(&db);
        
        // First create a group to find
        let new_group = GroupChatFactory::unique_fake_new_group_chat("find_test", user.id);
        let group_id = repository.insert(new_group.clone()).await.unwrap();

        // Act
        let result = repository.find_by_id(group_id).await;

        // Assert
        assert!(result.is_ok(), "Should find the group successfully");
        let found_group = result.unwrap();
        assert_eq!(found_group.id, group_id);
        assert_eq!(found_group.name, new_group.name);
        assert_eq!(found_group.description, new_group.description);
        assert_eq!(found_group.created_by, user.id);
        assert!(found_group.created_at <= found_group.updated_at);

        // Cleanup
        cleanup_group_chat(group_id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_not_found() {
        // Arrange
        let db = get_database().await;
        let repository = GroupChatRepository::new(&db);
        let non_existent_id = 99999;

        // Act
        let result = repository.find_by_id(non_existent_id).await;

        // Assert
        assert!(result.is_err(), "Should not find non-existent group");
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {
                // This is expected
            },
            other => panic!("Expected RowNotFound error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_negative_id() {
        // Arrange
        let db = get_database().await;
        let repository = GroupChatRepository::new(&db);
        let invalid_id = -1;

        // Act
        let result = repository.find_by_id(invalid_id).await;

        // Assert
        assert!(result.is_err(), "Should not find group with negative ID");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_multiple_groups() {
        // Arrange
        let (user, _) = create_test_user("multi_find_creator").await;
        let db = get_database().await;
        let repository = GroupChatRepository::new(&db);

        // Create multiple groups
        let group1 = GroupChatFactory::unique_fake_new_group_chat("multi_find_1", user.id);
        let group2 = GroupChatFactory::unique_fake_new_group_chat("multi_find_2", user.id);

        let group_id1 = repository.insert(group1.clone()).await.unwrap();
        let group_id2 = repository.insert(group2.clone()).await.unwrap();

        // Act
        let result1 = repository.find_by_id(group_id1).await;
        let result2 = repository.find_by_id(group_id2).await;

        // Assert
        assert!(result1.is_ok(), "Should find first group");
        assert!(result2.is_ok(), "Should find second group");

        let found_group1 = result1.unwrap();
        let found_group2 = result2.unwrap();

        assert_eq!(found_group1.id, group_id1);
        assert_eq!(found_group2.id, group_id2);
        assert_eq!(found_group1.name, group1.name);
        assert_eq!(found_group2.name, group2.name);
        assert_ne!(found_group1.id, found_group2.id);

        // Cleanup
        cleanup_group_chat(group_id1).await;
        cleanup_group_chat(group_id2).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_after_insert() {
        // Arrange
        let (user, _) = create_test_user("insert_find_creator").await;
        let db = get_database().await;
        let repository = GroupChatRepository::new(&db);

        // Use factory with specific data
        let new_group = GroupChatFactory::with_specific_creator(
            GroupChatFactory::with_name(
                GroupChatFactory::with_description(
                    GroupChatFactory::fake_new_group_chat(),
                    "Test description for find".to_string()
                ),
                "Test Group for Find".to_string()
            ),
            user.id
        );

        // Act
        let group_id = repository.insert(new_group.clone()).await.unwrap();
        let found_result = repository.find_by_id(group_id).await;

        // Assert
        assert!(found_result.is_ok(), "Should find newly inserted group");
        let found_group = found_result.unwrap();
        
        assert_eq!(found_group.id, group_id);
        assert_eq!(found_group.name, "Test Group for Find");
        assert_eq!(found_group.description, "Test description for find");
        assert_eq!(found_group.created_by, user.id);

        // Cleanup
        cleanup_group_chat(group_id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_concurrent_access() {
        // Arrange
        let (user, _) = create_test_user("concurrent_find_creator").await;
        let db = get_database().await;
        let repository = Arc::new(GroupChatRepository::new(&db));

        // Create a group first
        let new_group = GroupChatFactory::unique_fake_new_group_chat("concurrent_find", user.id);
        let group_id = repository.insert(new_group).await.unwrap();

        let mut handles = vec![];

        // Spawn concurrent find operations
        for i in 0..5 {
            let repo_clone = Arc::clone(&repository);
            
            let handle = tokio::spawn(async move {
                let result = repo_clone.find_by_id(group_id).await;
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
            assert_eq!(found_group.id, group_id);
        }

        // Cleanup
        cleanup_group_chat(group_id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_verify_timestamps() {
        // Arrange
        let (user, _) = create_test_user("timestamp_find_creator").await;
        let db = get_database().await;
        let repository = GroupChatRepository::new(&db);

        let new_group = GroupChatFactory::unique_fake_new_group_chat("timestamp_test", user.id);

        // Act
        let group_id = repository.insert(new_group).await.unwrap();
        let found_result = repository.find_by_id(group_id).await;

        // Assert
        assert!(found_result.is_ok(), "Should find group successfully");
        let found_group = found_result.unwrap();

        // Verify timestamps are reasonable
        assert!(found_group.created_at <= found_group.updated_at);
        assert!(found_group.created_at <= chrono::Utc::now());
        assert!(found_group.updated_at <= chrono::Utc::now());

        // Cleanup
        cleanup_group_chat(group_id).await;
        cleanup_user(user.email).await;
    }
}
