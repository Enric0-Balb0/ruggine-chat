
use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::dto::user_dto::UserRegisterDto;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::user_error::UserError;
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::factory::user_factory::UserFactory;

#[cfg(test)]
mod user_service_integration_tests {
    use ruggine_server::entity::user::UserStatus;

    use crate::get_database;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_create_user_success() {
        // Arrange: Set up a real database connection and service
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        let dto = UserFactory::unique_fake_user_register_dto("create_success");

        // Act: Call the create_user method
        let result = service.create_user(dto.clone()).await;

        // Assert: Verify the user was created successfully
        assert!(result.is_ok(), "Failed to create user: {:?}", result);
        let user_dto = result.unwrap();
        assert_eq!(user_dto.email, dto.email);
        assert_eq!(user_dto.first_name, dto.first_name);
        assert_eq!(user_dto.last_name, dto.last_name);
        assert_eq!(user_dto.username, dto.username);
        assert_eq!(user_dto.user_status, UserStatus::Active);
        assert!(user_dto.id > 0);

        // Cleanup: Delete the created user
        if let Err(e) = repository.delete_by_email(dto.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", dto.email, e);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_user_already_exists() {
        // Arrange: Create a user first, then try to create another with same email
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        // Create the first user using the factory
        let first_user = UserFactory::unique_fake_new_user("duplicate", UserStatus::Active);
        let insert_result = repository.insert(first_user.clone()).await;
        assert!(insert_result.is_ok(), "Failed to insert first user");

        // Try to create another user with the same email using UserRegisterDto
        let dto = UserRegisterDto {
            first_name: "Duplicate".into(),
            last_name: "Test".into(),
            username: "different_username".into(),
            email: first_user.email.clone(), // Same email as first user
            password: "password123".into(),
            birthday: chrono::NaiveDate::from_ymd_opt(1985, 3, 20).unwrap(),
            address: "789 Different St".to_string(),
            gender: ruggine_server::entity::user::Gender::Female,
        };

        // Act: Attempt to create a user with the same email
        let result = service.create_user(dto).await;

        // Assert: Should return UserAlreadyExists error
        assert!(result.is_err(), "Expected error but got success");
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::UserError(UserError::UserAlreadyExists(_))), "Expected UserAlreadyExists error, got: {:?}", error);

        // Cleanup: Delete the test user
        if let Err(e) = repository.delete_by_email(first_user.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", first_user.email, e);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_user_validates_data_integrity() {
        // Arrange: Set up a real database connection and service
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        let dto = UserFactory::unique_fake_user_register_dto("integrity");

        // Act: Create the user
        let result = service.create_user(dto.clone()).await;

        // Assert: Verify the user was created successfully
        assert!(result.is_ok(), "Failed to create user: {:?}", result);
        let user_dto = result.unwrap();

        // Verify data integrity - check that all fields are correctly stored
        assert_eq!(user_dto.first_name, dto.first_name);
        assert_eq!(user_dto.last_name, dto.last_name);
        assert_eq!(user_dto.username, dto.username);
        assert_eq!(user_dto.email, dto.email);
        assert_eq!(user_dto.user_status, UserStatus::Active);
        assert!(user_dto.created_at <= chrono::Utc::now());
        assert!(user_dto.updated_at <= chrono::Utc::now());

        // Verify password is properly hashed by trying to verify it
        let user_option = repository.find_by_email(dto.email.clone()).await;
        assert!(user_option.is_some(), "User not found in database");
        let user = user_option.unwrap();

        // Password should be hashed, not stored in plain text
        assert_ne!(user.password, dto.password, "Password should be hashed, not plain text");

        // But verification should work
        assert!(service.verify_password_internal(&user, &dto.password), "Password verification should work");

        // Cleanup: Delete the test user
        if let Err(e) = repository.delete_by_email(dto.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", dto.email, e);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_user_with_special_characters() {
        // Arrange: Test with special characters in names
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        let mut dto = UserFactory::unique_fake_user_register_dto("special");
        dto.first_name = "José María".into();
        dto.last_name = "García-López".into();

        // Act: Create the user
        let result = service.create_user(dto.clone()).await;

        // Assert: Verify the user was created successfully with special characters
        assert!(result.is_ok(), "Failed to create user with special characters: {:?}", result);
        let user_dto = result.unwrap();

        assert_eq!(user_dto.first_name, "José María");
        assert_eq!(user_dto.last_name, "García-López");
        assert_eq!(user_dto.username, dto.username);
        assert_eq!(user_dto.email, dto.email);

        // Cleanup: Delete the test user
        if let Err(e) = repository.delete_by_email(dto.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", dto.email, e);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_user_with_new_fields() {
        // Arrange: Test that new fields are properly stored
        let db = get_database().await;
        let service = UserService::new(&db);
        let repository = UserRepository::new(&db);

        let mut dto = UserFactory::unique_fake_user_register_dto("new_fields");
        dto.birthday = chrono::NaiveDate::from_ymd_opt(1995, 7, 15).unwrap();
        dto.address = "456 New Field Ave, Test City".to_string();
        dto.gender = ruggine_server::entity::user::Gender::Other;

        // Act: Create the user
        let result = service.create_user(dto.clone()).await;

        // Assert: Verify the user was created successfully
        assert!(result.is_ok(), "Failed to create user: {:?}", result);
        let user_dto = result.unwrap();

        // Verify that new fields are correctly returned in UserReadDto
        assert_eq!(user_dto.birthday, dto.birthday);
        assert_eq!(user_dto.address, dto.address);
        assert_eq!(user_dto.gender, dto.gender);
        assert_eq!(user_dto.is_online, false); // Should default to false

        // Also verify by fetching from database directly
        let user_option = repository.find_by_email(dto.email.clone()).await;
        assert!(user_option.is_some(), "User not found in database");
        let user = user_option.unwrap();

        assert_eq!(user.birthday, dto.birthday);
        assert_eq!(user.address, dto.address);
        assert_eq!(user.gender, dto.gender);
        assert_eq!(user.is_online, false); // Should default to false

        // Cleanup: Delete the test user
        if let Err(e) = repository.delete_by_email(dto.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", dto.email, e);
        }
    }
}