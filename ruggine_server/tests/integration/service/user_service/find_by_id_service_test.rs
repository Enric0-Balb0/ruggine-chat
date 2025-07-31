use ruggine_server::entity::user::UserStatus;
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::service::user_service::UserService;
use std::sync::Arc;

#[cfg(test)]
mod user_service_find_by_id_integration_tests {
    use ruggine_server::entity::user::UpdateUser;
    use ruggine_server::service::user_service::UserServiceTrait;
    use crate::common::cleanup_user;
    use crate::get_database;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_existing_user() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);

        let service = UserService::with_repo(Arc::new(repository));

        let new_user = UserFactory::unique_fake_new_user("find_by_id_test", UserStatus::Active);

        // Insert user first
        let inserted_id = service.user_repo().insert(new_user.clone()).await
            .expect("Failed to insert test user");

        // Act
        let result = service.find_by_id(inserted_id).await;

        // Assert
        assert!(result.is_ok());
        let found_user = result.unwrap();
        assert_eq!(found_user.id, inserted_id);
        assert_eq!(found_user.email, new_user.email);
        assert_eq!(found_user.username, new_user.username);
        assert_eq!(found_user.first_name, new_user.first_name);
        assert_eq!(found_user.last_name, new_user.last_name);
        assert_eq!(found_user.user_status, new_user.user_status);

        // Cleanup
        cleanup_user(found_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_non_existent_user() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let service = UserService::with_repo(Arc::new(repository));

        let non_existent_id = 99999; // Very unlikely to exist

        // Act
        let result = service.find_by_id(non_existent_id).await;

        // Assert
        assert!(result.is_err());
        // Should return a database error for row not found
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_multiple_users() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let service = UserService::with_repo(Arc::new(repository));

        // Create multiple users using factory
        let user1 = UserFactory::unique_fake_new_user("multi_find_1", UserStatus::Active);
        let user2 = UserFactory::unique_fake_new_user("multi_find_2", UserStatus::Deleted);
        let user3 = UserFactory::unique_fake_new_user("multi_find_3", UserStatus::Active);

        // Insert all users
        let id1 = service.user_repo().insert(user1.clone()).await.expect("Failed to insert user1");
        let id2 = service.user_repo().insert(user2.clone()).await.expect("Failed to insert user2");
        let id3 = service.user_repo().insert(user3.clone()).await.expect("Failed to insert user3");

        // Act - Find each user
        let result1 = service.find_by_id(id1).await;
        let result2 = service.find_by_id(id2).await;
        let result3 = service.find_by_id(id3).await;

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());

        let found1 = result1.unwrap();
        let found2 = result2.unwrap();
        let found3 = result3.unwrap();

        // Verify data integrity
        assert_eq!(found1.email, user1.email);
        assert_eq!(found1.user_status, UserStatus::Active);

        assert_eq!(found2.email, user2.email);
        assert_eq!(found2.user_status, UserStatus::Deleted);

        assert_eq!(found3.email, user3.email);
        assert_eq!(found3.user_status, UserStatus::Active);

        // Cleanup
        cleanup_user(user1.email).await;
        cleanup_user(user2.email).await;
        cleanup_user(user3.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_after_insert_and_update() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let service = UserService::with_repo(Arc::new(repository));

        let new_user = UserFactory::unique_fake_new_user("update_find_test", UserStatus::Active);

        // Insert user
        let inserted_id = service.user_repo().insert(new_user.clone()).await
            .expect("Failed to insert test user");

        let original_user = service.find_by_id(inserted_id).await.unwrap();

        // Update user status
        let update_user = UserFactory::fake_user_update_dto_from_user_read_dto(
            &original_user,
            "update_find_test",
        );
        service.user_repo().update_profile(inserted_id, UpdateUser::from_dto(update_user.clone())).await
            .expect("Failed to update user");

        // Act - Find updated user
        let result = service.find_by_id(inserted_id).await;

        // Assert
        assert!(result.is_ok());
        let found_user = result.unwrap();
        assert_eq!(found_user.id, original_user.id);
        assert_eq!(found_user.user_status, original_user.user_status);
        assert_eq!(found_user.email, original_user.email); // Other fields unchanged
        assert_eq!(found_user.username, original_user.username);

        assert_eq!(found_user.first_name, update_user.first_name.unwrap());
        assert_eq!(found_user.last_name, update_user.last_name.unwrap());

        // Cleanup
        cleanup_user(found_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_with_different_user_statuses() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let service = UserService::with_repo(Arc::new(repository));

        // Create users with different statuses
        let active_user = UserFactory::unique_fake_new_user("status_active", UserStatus::Active);
        let inactive_user = UserFactory::unique_fake_new_user("status_inactive", UserStatus::Deleted);

        let active_id = service.user_repo().insert(active_user.clone()).await
            .expect("Failed to insert active user");
        let inactive_id = service.user_repo().insert(inactive_user.clone()).await
            .expect("Failed to insert inactive user");

        // Act
        let active_result = service.find_by_id(active_id).await;
        let inactive_result = service.find_by_id(inactive_id).await;

        // Assert
        assert!(active_result.is_ok());
        assert!(inactive_result.is_ok());

        let found_active = active_result.unwrap();
        let found_inactive = inactive_result.unwrap();

        assert_eq!(found_active.user_status, UserStatus::Active);
        assert_eq!(found_inactive.user_status, UserStatus::Deleted);

        // Cleanup
        cleanup_user(found_active.email).await;
        cleanup_user(found_inactive.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_concurrent_access() {
        // Arrange
        let db = get_database().await;
        let repository = Arc::new(UserRepository::new(&db));
        let service = UserService::with_repo(repository.clone());

        // Create a test user
        let new_user = UserFactory::unique_fake_new_user("concurrent_find", UserStatus::Active);
        let inserted_id = repository.insert(new_user.clone()).await
            .expect("Failed to insert test user");

        // Act - Multiple concurrent find operations
        let mut handles = vec![];
        for i in 0..10 {
            let service_clone = service.clone();
            let handle = tokio::spawn(async move {
                service_clone.find_by_id(inserted_id).await
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        let results = futures::future::join_all(handles).await;

        // Assert
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_ok(), "Task {} panicked", i);
            let find_result = result.as_ref().unwrap();
            assert!(find_result.is_ok(), "Find operation {} failed", i);

            let user = find_result.as_ref().unwrap();
            assert_eq!(user.id, inserted_id);
            assert_eq!(user.email, new_user.email);

            // Cleanup
            cleanup_user(user.email.clone()).await;
        }
    }
}