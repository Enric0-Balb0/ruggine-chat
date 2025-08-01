use std::sync::Arc;
use ruggine_server::repository::group_chat_repository::{GroupChatRepository, GroupChatRepositoryTrait};
use ruggine_server::factory::group_chat_factory::GroupChatFactory;
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::entity::user::UserStatus;
use crate::common::{cleanup_user, create_test_user, cleanup_group_chat};

#[cfg(test)]
mod group_chat_repository_integration_tests {
    use crate::get_database;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_group_chat_success() {
        // Arrange
        let (user, _) = create_test_user("group_creator").await;
        let db = get_database().await;
        let repository = GroupChatRepository::new(&db);
        let new_group = GroupChatFactory::unique_fake_new_group_chat("test_group", user.id);

        // Act
        let result = repository.insert(new_group.clone()).await;

        // Assert
        assert!(result.is_ok(), "Group insert should succeed");
        let group_id = result.unwrap();
        assert!(group_id > 0, "Group ID should be positive");

        // Cleanup
        cleanup_group_chat(group_id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_group_chat_duplicate_name() {
        // Arrange
        let (user, _) = create_test_user("group_creator_dup").await;
        let db = get_database().await;
        let repository = GroupChatRepository::new(&db);

        let group1 = GroupChatFactory::unique_fake_new_group_chat("duplicate_test", user.id);
        let mut group2 = GroupChatFactory::unique_fake_new_group_chat("duplicate_test", user.id);
        group2.name = group1.name.clone(); // Same name

        // Act
        let first_insert = repository.insert(group1.clone()).await;
        let second_insert = repository.insert(group2.clone()).await;

        // Assert
        assert!(first_insert.is_ok(), "First group insert should succeed");
        let first_group_id = first_insert.unwrap();
        assert!(second_insert.is_ok(), "Second group insert should succeed");
        let second_group_id = second_insert.unwrap();

        assert_ne!(first_group_id, second_group_id, "Same name, but group ids should not be equal");

        // Cleanup
        cleanup_group_chat(first_group_id).await;
        cleanup_group_chat(second_group_id).await;
        
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_group_chat_invalid_creator() {
        // Arrange
        let db = get_database().await;
        let repository = GroupChatRepository::new(&db);
        let invalid_user_id = -1; // Non-existent user ID
        let new_group = GroupChatFactory::unique_fake_new_group_chat("invalid_creator", invalid_user_id);

        // Act
        let result = repository.insert(new_group).await;

        // Assert
        assert!(result.is_err(), "Insert should fail with invalid creator");
        
        // Verify it's a foreign key constraint error
        match result.unwrap_err() {
            sqlx::Error::Database(db_err) => {
                // PostgreSQL foreign key violation error code is "23503"
                assert!(db_err.code().map_or(false, |code| code == "23503"));
            },
            other => panic!("Expected database error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_repository_concurrent_group_creation() {
        use tokio::sync::Mutex;

        // Arrange
        let (user, _) = create_test_user("concurrent_creator").await;
        let db = get_database().await;
        let repository = Arc::new(GroupChatRepository::new(&db));
        let group_ids_to_cleanup = Arc::new(Mutex::new(vec![]));

        let mut handles = vec![];

        // Spawn concurrent group creations
        for i in 0..5 {
            let repo_clone = Arc::clone(&repository);
            let ids_clone = Arc::clone(&group_ids_to_cleanup);

            let handle = tokio::spawn(async move {
                let new_group = GroupChatFactory::unique_fake_new_group_chat(&format!("concurrent_{}", i), user.id);
                let result = repo_clone.insert(new_group).await;
                
                if let Ok(group_id) = result {
                    let mut guard = ids_clone.lock().await;
                    guard.push(group_id);
                }
                
                result
            });
            handles.push(handle);
        }

        // Wait for all inserts
        let results = futures::future::join_all(handles).await;

        // Assert
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Task {i} panicked");
            assert!(result.as_ref().unwrap().is_ok(), "Group insert {i} failed");
        }

        // Cleanup
        let group_ids = group_ids_to_cleanup.lock().await.clone();
        
        let cleanup_handle = tokio::task::spawn(async move {
            for group_id in group_ids {
                cleanup_group_chat(group_id).await;
            }
            cleanup_user(user.email).await;
        });

        // Wait for cleanup to finish
        if let Err(e) = cleanup_handle.await {
            eprintln!("Cleanup task panicked: {:?}", e);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_group_with_long_name() {
        // Arrange
        let (user, _) = create_test_user("long_name_creator").await;
        let db = get_database().await;
        let repository = GroupChatRepository::new(&db);
        
        let mut group = GroupChatFactory::fake_new_group_chat();
        group.created_by = user.id;
        group.name = "a".repeat(256); // Maximum allowed length

        // Act
        let result = repository.insert(group).await;

        // Assert
        assert!(result.is_ok(), "Insert with max length name should succeed");
        let group_id = result.unwrap();

        // Cleanup
        cleanup_group_chat(group_id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_group_with_long_description() {
        // Arrange
        let (user, _) = create_test_user("long_desc_creator").await;
        let db = get_database().await;
        let repository = GroupChatRepository::new(&db);
        
        let mut group = GroupChatFactory::fake_new_group_chat();
        group.created_by = user.id;
        group.description = "a".repeat(1024); // Maximum allowed length

        // Act
        let result = repository.insert(group).await;

        // Assert
        assert!(result.is_ok(), "Insert with max length description should succeed");
        let group_id = result.unwrap();

        // Cleanup
        cleanup_group_chat(group_id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_multiple_groups_same_creator() {
        // Arrange
        let (user, _) = create_test_user("multi_group_creator").await;
        let db = get_database().await;
        let repository = GroupChatRepository::new(&db);
        
        let group1 = GroupChatFactory::unique_fake_new_group_chat("multi_1", user.id);
        let group2 = GroupChatFactory::unique_fake_new_group_chat("multi_2", user.id);
        let group3 = GroupChatFactory::unique_fake_new_group_chat("multi_3", user.id);

        // Act
        let result1 = repository.insert(group1).await;
        let result2 = repository.insert(group2).await;
        let result3 = repository.insert(group3).await;

        // Assert
        assert!(result1.is_ok(), "First group insert should succeed");
        assert!(result2.is_ok(), "Second group insert should succeed");
        assert!(result3.is_ok(), "Third group insert should succeed");

        let group_id1 = result1.unwrap();
        let group_id2 = result2.unwrap();
        let group_id3 = result3.unwrap();

        // Verify all IDs are different
        assert_ne!(group_id1, group_id2);
        assert_ne!(group_id2, group_id3);
        assert_ne!(group_id1, group_id3);

        // Cleanup
        cleanup_group_chat(group_id1).await;
        cleanup_group_chat(group_id2).await;
        cleanup_group_chat(group_id3).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_group_with_factory_utilities() {
        // Arrange
        let (user, _) = create_test_user("factory_util_creator").await;
        let db = get_database().await;
        let repository = GroupChatRepository::new(&db);
        
        let base_group = GroupChatFactory::fake_new_group_chat();
        let custom_group = GroupChatFactory::with_specific_creator(
            GroupChatFactory::with_name(
                GroupChatFactory::with_description(
                    base_group, 
                    "Custom factory description".to_string()
                ),
                "Custom Factory Group".to_string()
            ),
            user.id
        );

        // Act
        let result = repository.insert(custom_group.clone()).await;

        // Assert
        assert!(result.is_ok(), "Factory-created group insert should succeed");
        let group_id = result.unwrap();
        assert!(group_id > 0);

        // Verify the custom values were used
        assert_eq!(custom_group.name, "Custom Factory Group");
        assert_eq!(custom_group.description, "Custom factory description");
        assert_eq!(custom_group.created_by, user.id);

        // Cleanup
        cleanup_group_chat(group_id).await;
        cleanup_user(user.email).await;
    }
}