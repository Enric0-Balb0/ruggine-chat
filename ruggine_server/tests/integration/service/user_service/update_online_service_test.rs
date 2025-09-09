use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::dto::user_dto::UpdateOnlineDto;
use ruggine_server::entity::user::User;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::factory::user_factory::UserFactory;
use crate::common::cleanup_user_by_email;

#[cfg(test)]
mod user_service_update_online_integration_tests {
    use crate::get_database;
    use super::*;

    /// Helper function to create a real user in the database for testing
    async fn create_test_user_for_update_online(prefix: &str) -> User {
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        let dto = UserFactory::unique_fake_user_register_dto(prefix);
        let create_result = service.create_user(dto.clone()).await;
        assert!(create_result.is_ok(), "Failed to create user for update online test");

        // Get the created user from database
        let user_option = repository.find_by_email(dto.email.clone()).await.unwrap();
        assert!(user_option.is_some(), "User not found in database");
        user_option.unwrap()
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_online_success_set_to_true() {
        // Arrange: Create a user in the database (initially offline)
        let user = create_test_user_for_update_online("service_online_true").await;
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        // Verify user is initially offline
        assert!(!user.is_online, "User should initially be offline");

        let update_dto = UserFactory::fake_update_online_dto_true();

        // Act: Call the update_online method
        let result = service.update_online(user.id, update_dto).await;

        // Assert: Verify the update was successful
        assert!(result.is_ok(), "Update online should succeed: {:?}", result);

        // Verify the user is now online by fetching from database
        let updated_user = repository.find(user.id).await;
        assert!(updated_user.is_ok(), "Failed to fetch updated user");
        let updated_user = updated_user.unwrap();

        assert!(updated_user.is_online, "User should be online after update");
        assert!(updated_user.updated_at > user.updated_at, "updated_at should be refreshed");

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_online_success_set_to_false() {
        // Arrange: Create a user and set them online first
        let user = create_test_user_for_update_online("service_online_false").await;
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        // First set user to online
        let set_online_dto = UserFactory::fake_update_online_dto_true();
        let set_online_result = service.update_online(user.id, set_online_dto).await;
        assert!(set_online_result.is_ok(), "Failed to initially set user online");

        // Verify user is online
        let online_user = repository.find(user.id).await.unwrap();
        assert!(online_user.is_online, "User should be online");

        let update_dto = UserFactory::fake_update_online_dto_false();

        // Act: Set user to offline
        let result = service.update_online(user.id, update_dto).await;

        // Assert: Verify the update was successful
        assert!(result.is_ok(), "Update online to false should succeed: {:?}", result);

        // Verify the user is now offline by fetching from database
        let updated_user = repository.find(user.id).await;
        assert!(updated_user.is_ok(), "Failed to fetch updated user");
        let updated_user = updated_user.unwrap();

        assert!(!updated_user.is_online, "User should be offline after update");
        assert!(updated_user.updated_at > online_user.updated_at, "updated_at should be refreshed");

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_online_toggle_multiple_times() {
        // Arrange: Create a user in the database
        let user = create_test_user_for_update_online("service_toggle_online").await;
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        let mut last_updated_at = user.updated_at;

        // Act & Assert: Toggle online status multiple times using service
        for (iteration, expected_status) in [(1, true), (2, false), (3, true), (4, false)].iter() {
            let update_dto = UserFactory::fake_update_online_dto(*expected_status);
            let result = service.update_online(user.id, update_dto).await;

            assert!(result.is_ok(), "Failed to update online status on iteration {}: {:?}", iteration, result);

            // Verify the status was updated correctly
            let updated_user = repository.find(user.id).await.unwrap();
            assert_eq!(updated_user.is_online, *expected_status,
                       "User online status should be {} on iteration {}", expected_status, iteration);
            assert!(updated_user.updated_at > last_updated_at,
                    "updated_at should be refreshed on iteration {}", iteration);

            last_updated_at = updated_user.updated_at;
        }

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_online_nonexistent_user() {
        // Arrange: Use a non-existent user ID
        let db = get_database().await;
        let service = UserService::new(&db);
        let nonexistent_user_id = -999;

        let update_dto = UserFactory::fake_update_online_dto_true();

        // Act: Try to update online status for non-existent user
        let result = service.update_online(nonexistent_user_id, update_dto).await;

        // Assert: Service should handle this gracefully 
        // The current implementation returns Ok(()) even for non-existent users
        // This is consistent with the repository behavior
        assert!(result.is_ok(), "Service should handle non-existent user gracefully");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_online_same_status_no_error() {
        // Arrange: Create a user in the database (initially offline)
        let user = create_test_user_for_update_online("service_same_status").await;
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        // Verify user is initially offline
        assert!(!user.is_online, "User should initially be offline");

        let update_dto = UserFactory::fake_update_online_dto_false();

        // Act: Set user to offline again (same status)
        let result = service.update_online(user.id, update_dto).await;

        // Assert: Operation should succeed
        assert!(result.is_ok(), "Update online with same status should succeed: {:?}", result);

        // Verify the updated_at timestamp was still updated
        let updated_user = repository.find(user.id).await.unwrap();
        assert!(!updated_user.is_online, "User should still be offline");
        assert!(updated_user.updated_at > user.updated_at, "updated_at should be refreshed even for same status");

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_online_concurrent_updates() {
        // Arrange: Create a user in the database
        let user = create_test_user_for_update_online("service_concurrent").await;
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        // Act: Perform concurrent updates
        let service1 = service.clone();
        let service2 = service.clone();
        let user_id = user.id;

        let update_dto1 = UserFactory::fake_update_online_dto_true();
        let update_dto2 = UserFactory::fake_update_online_dto_false();

        let task1 = tokio::spawn(async move {
            service1.update_online(user_id, update_dto1).await
        });

        let task2 = tokio::spawn(async move {
            service2.update_online(user_id, update_dto2).await
        });

        // Wait for both tasks to complete
        let (result1, result2) = tokio::join!(task1, task2);

        // Assert: Both operations should succeed
        assert!(result1.is_ok(), "First concurrent update should succeed");
        assert!(result2.is_ok(), "Second concurrent update should succeed");
        assert!(result1.unwrap().is_ok(), "First update result should be Ok");
        assert!(result2.unwrap().is_ok(), "Second update result should be Ok");

        // Verify final state (one of the updates should have won)
        let final_user = repository.find(user.id).await.unwrap();
        // We can't predict which update wins, but the user should have an updated timestamp
        assert!(final_user.updated_at > user.updated_at, "updated_at should be refreshed");

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_online_using_factory_methods() {
        // Arrange: Create a user and test different factory methods
        let user = create_test_user_for_update_online("service_factory_methods").await;
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        // Test using fake_update_online_dto(true)
        let update_dto_true = UserFactory::fake_update_online_dto(true);
        let result_true = service.update_online(user.id, update_dto_true).await;
        assert!(result_true.is_ok(), "Update with factory method (true) should succeed");

        let user_after_true = repository.find(user.id).await.unwrap();
        assert!(user_after_true.is_online, "User should be online after factory true method");

        // Test using fake_update_online_dto(false)
        let update_dto_false = UserFactory::fake_update_online_dto(false);
        let result_false = service.update_online(user.id, update_dto_false).await;
        assert!(result_false.is_ok(), "Update with factory method (false) should succeed");

        let user_after_false = repository.find(user.id).await.unwrap();
        assert!(!user_after_false.is_online, "User should be offline after factory false method");

        // Test using specific factory methods
        let update_dto_specific_true = UserFactory::fake_update_online_dto_true();
        let result_specific_true = service.update_online(user.id, update_dto_specific_true).await;
        assert!(result_specific_true.is_ok(), "Update with specific factory true method should succeed");

        let update_dto_specific_false = UserFactory::fake_update_online_dto_false();
        let result_specific_false = service.update_online(user.id, update_dto_specific_false).await;
        assert!(result_specific_false.is_ok(), "Update with specific factory false method should succeed");

        let final_user = repository.find(user.id).await.unwrap();
        assert!(!final_user.is_online, "User should be offline after final update");

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_online_preserves_user_data_integrity() {
        // Arrange: Create a user with specific data
        let user = create_test_user_for_update_online("service_integrity").await;
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        // Store original values to verify they don't change
        let original_email = user.email.clone();
        let original_username = user.username.clone();
        let original_first_name = user.first_name.clone();
        let original_last_name = user.last_name.clone();
        let original_user_status = user.user_status.clone();
        let original_user_type = user.user_type.clone();
        let original_birthday = user.birthday;
        let original_address = user.address.clone();
        let original_gender = user.gender.clone();
        let original_created_at = user.created_at;

        let update_dto = UserFactory::fake_update_online_dto_true();

        // Act: Update online status
        let result = service.update_online(user.id, update_dto).await;

        // Assert: Operation should succeed
        assert!(result.is_ok(), "Update online should succeed: {:?}", result);

        // Verify all other fields remain unchanged
        let updated_user = repository.find(user.id).await.unwrap();

        assert_eq!(updated_user.email, original_email);
        assert_eq!(updated_user.username, original_username);
        assert_eq!(updated_user.first_name, original_first_name);
        assert_eq!(updated_user.last_name, original_last_name);
        assert_eq!(updated_user.user_status, original_user_status);
        assert_eq!(updated_user.user_type, original_user_type);
        assert_eq!(updated_user.birthday, original_birthday);
        assert_eq!(updated_user.address, original_address);
        assert_eq!(updated_user.gender, original_gender);
        assert_eq!(updated_user.created_at, original_created_at);

        // Only is_online and updated_at should change
        assert!(updated_user.is_online);
        assert!(updated_user.updated_at > user.updated_at);

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }
}
