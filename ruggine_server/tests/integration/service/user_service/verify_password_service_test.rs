
use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::factory::user_factory::UserFactory;

#[cfg(test)]
mod user_service_verify_password_integration_tests {

    use crate::get_database;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_verify_password_correct() {
        // Arrange: Create a user in the database with a known password
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        let password = "testpassword123";
        let mut dto = UserFactory::unique_fake_user_register_dto("verify_correct");
        dto.password = password.into();

        // Create the user
        let create_result = service.create_user(dto.clone()).await;
        assert!(create_result.is_ok(), "Failed to create user for verification test");

        // Retrieve the user from database
        let user_option = repository.find_by_email(dto.email.clone()).await.unwrap();
        assert!(user_option.is_some(), "User not found in database");
        let user = user_option.unwrap();

        // Act: Verify the correct password
        let result = service.verify_password_internal(&user, password);

        // Assert: Should return true for correct password
        assert!(result, "Expected password verification to succeed");

        // Cleanup: Delete the test user
        if let Err(e) = repository.delete_by_email(dto.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", dto.email, e);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_verify_password_incorrect() {
        // Arrange: Create a user in the database with a known password
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        let correct_password = "testpassword123";
        let incorrect_password = "wrongpassword";
        let mut dto = UserFactory::unique_fake_user_register_dto("verify_incorrect");
        dto.password = correct_password.into();

        // Create the user
        let create_result = service.create_user(dto.clone()).await;
        assert!(create_result.is_ok(), "Failed to create user for verification test");

        // Retrieve the user from database
        let user_option = repository.find_by_email(dto.email.clone()).await.unwrap();
        assert!(user_option.is_some(), "User not found in database");
        let user = user_option.unwrap();

        // Act: Verify an incorrect password
        let result = service.verify_password_internal(&user, incorrect_password);

        // Assert: Should return false for incorrect password
        assert!(!result, "Expected password verification to fail");

        // Cleanup: Delete the test user
        if let Err(e) = repository.delete_by_email(dto.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", dto.email, e);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_verify_password_empty_password() {
        // Arrange: Create a user in the database with a known password
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        let correct_password = "testpassword123";
        let mut dto = UserFactory::unique_fake_user_register_dto("verify_empty");
        dto.password = correct_password.into();

        // Create the user
        let create_result = service.create_user(dto.clone()).await;
        assert!(
            create_result.is_ok(),
            "Failed to create user for verification test: {:?}",
            create_result.unwrap_err()
        );

        // Retrieve the user from database
        let user_option = repository.find_by_email(dto.email.clone()).await.unwrap();
        assert!(user_option.is_some(), "User not found in database");
        let user = user_option.unwrap();

        // Act: Verify an empty password
        let result = service.verify_password_internal(&user, "");

        // Assert: Should return false for empty password
        assert!(!result, "Expected password verification to fail for empty password");

        // Cleanup: Delete the test user
        if let Err(e) = repository.delete_by_email(dto.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", dto.email, e);
        }
    }
}