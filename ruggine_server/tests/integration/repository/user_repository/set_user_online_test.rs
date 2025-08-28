use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::entity::user::User;
use crate::common::{get_database, create_test_user, cleanup_user_by_email};

#[cfg(test)]
mod set_user_online_integration_tests {
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_set_user_online_success_set_to_true() {
        // Arrange: Create a test user who is initially offline
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let (user, _password) = create_test_user("set_online_true").await;
        
        // Verify user is initially offline (default value)
        assert!(!user.is_online, "User should initially be offline");

        // Act: Set user to online
        let result = repository.update_online(user.id, true).await;

        // Assert: Operation should succeed
        assert!(result.is_ok(), "Failed to set user online: {:?}", result);

        // Verify the user is now online by fetching from database
        let updated_user = repository.find(user.id).await;
        assert!(updated_user.is_ok(), "Failed to fetch updated user");
        let updated_user = updated_user.unwrap();
        
        assert!(updated_user.is_online, "User should be online after update");
        assert!(updated_user.updated_at > user.updated_at, "updated_at should be refreshed");
        
        // Verify other fields remain unchanged
        assert_eq!(updated_user.id, user.id);
        assert_eq!(updated_user.email, user.email);
        assert_eq!(updated_user.username, user.username);
        assert_eq!(updated_user.first_name, user.first_name);
        assert_eq!(updated_user.last_name, user.last_name);

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_set_user_online_success_set_to_false() {
        // Arrange: Create a test user and set them online first
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let (user, _password) = create_test_user("set_online_false").await;
        
        // First set user to online
        let set_online_result = repository.update_online(user.id, true).await;
        assert!(set_online_result.is_ok(), "Failed to initially set user online");
        
        // Verify user is online
        let online_user = repository.find(user.id).await.unwrap();
        assert!(online_user.is_online, "User should be online");

        // Act: Set user to offline
        let result = repository.update_online(user.id, false).await;

        // Assert: Operation should succeed
        assert!(result.is_ok(), "Failed to set user offline: {:?}", result);

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
    async fn test_set_user_online_toggle_multiple_times() {
        // Arrange: Create a test user
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let (user, _password) = create_test_user("toggle_online").await;
        
        let mut last_updated_at = user.updated_at;

        // Act & Assert: Toggle online status multiple times
        for (iteration, expected_status) in [(1, true), (2, false), (3, true), (4, false)].iter() {
            let result = repository.update_online(user.id, *expected_status).await;
            assert!(result.is_ok(), "Failed to set user online status on iteration {}: {:?}", iteration, result);

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
    async fn test_set_user_online_nonexistent_user() {
        // Arrange: Use a non-existent user ID
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let nonexistent_user_id = -999;

        // Act: Try to set online status for non-existent user
        let result = repository.update_online(nonexistent_user_id, true).await;

        // Assert: Operation should succeed (no error) but no rows affected
        // The method returns Ok(()) regardless of whether the user exists
        assert!(result.is_ok(), "Operation should succeed even for non-existent user");
        
        // The method logs when no user is found, but doesn't return an error
        // This is consistent with the current implementation that logs but doesn't fail
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_set_user_online_same_status_no_op() {
        // Arrange: Create a test user
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let (user, _password) = create_test_user("same_status").await;
        
        // Verify user is initially offline
        assert!(!user.is_online, "User should initially be offline");

        // Act: Set user to offline again (no change)
        let result = repository.update_online(user.id, false).await;

        // Assert: Operation should succeed
        assert!(result.is_ok(), "Failed to set user offline (same status): {:?}", result);

        // Verify the updated_at timestamp was still updated even though status didn't change
        let updated_user = repository.find(user.id).await.unwrap();
        assert!(!updated_user.is_online, "User should still be offline");
        assert!(updated_user.updated_at > user.updated_at, "updated_at should be refreshed even for same status");

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_set_user_online_concurrent_updates() {
        // Arrange: Create a test user
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let (user, _password) = create_test_user("concurrent_updates").await;

        // Act: Perform concurrent updates
        let repo1 = repository.clone();
        let repo2 = repository.clone();
        let user_id = user.id;

        let task1 = tokio::spawn(async move {
            repo1.update_online(user_id, true).await
        });

        let task2 = tokio::spawn(async move {
            repo2.update_online(user_id, false).await
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
    async fn test_set_user_online_preserves_other_fields() {
        // Arrange: Create a test user with specific data
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let (user, _password) = create_test_user("preserve_fields").await;
        
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

        // Act: Set user online
        let result = repository.update_online(user.id, true).await;

        // Assert: Operation should succeed
        assert!(result.is_ok(), "Failed to set user online: {:?}", result);

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
