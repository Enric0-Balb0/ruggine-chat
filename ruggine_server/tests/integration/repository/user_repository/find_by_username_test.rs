use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::factory::user_factory::UserFactory;

#[cfg(test)]
mod user_repository_integration_tests {
    use ruggine_server::entity::user::UserStatus;
    use crate::get_database;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_find_and_delete_by_username() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);

        let new_user = UserFactory::unique_fake_new_user("insertfindusername", UserStatus::Active);

        // Act
        let insert_result = repository.insert(new_user.clone()).await;

        // Assert
        assert!(insert_result.is_ok(), "Failed to insert user: {:?}", insert_result);
        let user_id = insert_result.unwrap();
        assert!(user_id > 0);

        // Test find_by_username
        let found_user_result = repository.find_by_username(new_user.username.clone()).await;
        assert!(found_user_result.is_ok(), "Database query should succeed");
        
        let found_user = found_user_result.unwrap();
        assert!(found_user.is_some(), "User should be found by username");

        let user = found_user.unwrap();
        assert_eq!(user.username, new_user.username);
        assert_eq!(user.email, new_user.email);
        assert_eq!(user.first_name, new_user.first_name);
        assert_eq!(user.last_name, new_user.last_name);

        // Cleanup
        if let Err(e) = repository.delete_by_email(new_user.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", new_user.email, e);
        }
        let cleanup_check = repository.find_by_username(new_user.username.clone()).await;
        assert!(cleanup_check.is_ok());
        assert!(cleanup_check.unwrap().is_none(), "User should be deleted");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_not_found() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);

        let (_, nonexistent_username, _) = UserFactory::get_unique_user_information("nonexistent");

        // Act
        let result = repository.find_by_username(nonexistent_username).await;

        // Assert
        assert!(result.is_ok(), "Database query should succeed even when user not found");
        assert!(result.unwrap().is_none(), "Expected no user to be found");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_empty_string() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);

        // Act
        let result = repository.find_by_username("".to_string()).await;

        // Assert
        assert!(result.is_ok(), "Database query should succeed even with empty string");
        assert!(result.unwrap().is_none(), "Expected no user to be found with empty username");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_case_sensitive() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);

        let new_user = UserFactory::unique_fake_new_user("casesensitive", UserStatus::Active);
        let original_username = new_user.username.clone();

        // Insert user
        let insert_result = repository.insert(new_user.clone()).await;
        assert!(insert_result.is_ok(), "Failed to insert user: {:?}", insert_result);

        // Act & Assert - Test exact match
        let found_exact = repository.find_by_username(original_username.clone()).await;
        assert!(found_exact.is_ok());
        assert!(found_exact.unwrap().is_some(), "Should find user with exact username match");

        // Test case sensitivity - uppercase
        let uppercase_username = original_username.to_uppercase();
        let found_uppercase = repository.find_by_username(uppercase_username).await;
        assert!(found_uppercase.is_ok());
        assert!(found_uppercase.unwrap().is_none(), "Should not find user with uppercase username");

        // Test case sensitivity - mixed case
        let mixed_case_username = original_username.chars()
            .enumerate()
            .map(|(i, c)| if i % 2 == 0 { c.to_uppercase().collect::<String>() } else { c.to_lowercase().collect::<String>() })
            .collect::<String>();
        let found_mixed = repository.find_by_username(mixed_case_username).await;
        assert!(found_mixed.is_ok());
        assert!(found_mixed.unwrap().is_none(), "Should not find user with mixed case username");

        // Cleanup
        if let Err(e) = repository.delete_by_email(new_user.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", new_user.email, e);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_special_characters() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);

        // Create user with special characters in username
        let mut new_user = UserFactory::unique_fake_new_user("specialchars", UserStatus::Active);
        new_user.username = format!("test_user-{}.special", chrono::Utc::now().timestamp_millis());

        // Insert user
        let insert_result = repository.insert(new_user.clone()).await;
        assert!(insert_result.is_ok(), "Failed to insert user with special characters: {:?}", insert_result);

        // Act
        let found_user_result = repository.find_by_username(new_user.username.clone()).await;

        // Assert
        assert!(found_user_result.is_ok(), "Database query should succeed");
        let found_user = found_user_result.unwrap();
        assert!(found_user.is_some(), "Should find user with special characters in username");
        
        let user = found_user.unwrap();
        assert_eq!(user.username, new_user.username);

        // Cleanup
        if let Err(e) = repository.delete_by_email(new_user.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", new_user.email, e);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_sql_injection_protection() {
        // Arrange
        let db = get_database().await;
        let repository = UserRepository::new(&db);

        // Test with potential SQL injection strings
        let malicious_usernames = vec![
            "'; DROP TABLE \"user\"; --".to_string(),
            "' OR '1'='1".to_string(),
            "'; SELECT * FROM \"user\"; --".to_string(),
            "admin' --".to_string(),
            "' UNION SELECT * FROM \"user\" --".to_string(),
        ];

        for malicious_username in malicious_usernames {
            // Act
            let result = repository.find_by_username(malicious_username.clone()).await;

            // Assert
            assert!(result.is_ok(), "Database query should succeed and not cause SQL injection for: {}", malicious_username);
            assert!(result.unwrap().is_none(), "Should not find user with malicious username: {}", malicious_username);
        }
    }
}
