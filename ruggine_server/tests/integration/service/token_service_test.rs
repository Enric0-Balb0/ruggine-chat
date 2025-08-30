use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::factory::user_factory::UserFactory;
use chrono::Utc;
use std::sync::atomic::{AtomicU64, Ordering};

// Global counter to ensure unique JWT secrets across all tests
static JWT_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Helper function to generate unique JWT secret for each test
fn get_unique_jwt_secret(test_name: &str) -> String {
    let counter = JWT_COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    format!("test_secret_{}_{}_{}_{}", test_name, counter, timestamp, std::process::id())
}

#[cfg(test)]
mod token_service_integration_tests {
    use ruggine_server::service::token_service::{TokenService, TokenServiceTrait};
    use crate::get_database;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_generate_token_with_real_user() {
        // Arrange: Create a real user in the database
        let db = get_database().await;
        let user_service = UserService::new(&db);
        let repository = UserRepository::new(&db);
        
        // Set up unique JWT secret for token generation
        let jwt_secret = get_unique_jwt_secret("integration");
        let token_service = TokenService::new(jwt_secret);
        
        // Create a unique user using UserFactory
        let user_dto = UserFactory::unique_fake_user_register_dto("token_gen");
        let create_result = user_service.create_user(user_dto.clone()).await;
        assert!(create_result.is_ok(), "Failed to create user for token test");
        
        // Get the created user from database
        let user_option = repository.find_by_email(user_dto.email.clone()).await;
        assert!(user_option.is_some(), "User not found in database");
        let user = user_option.unwrap();

        // Act: Generate token for the real user
        let result = token_service.generate_token(user.clone());

        // Assert: Verify token was generated successfully
        assert!(result.is_ok(), "Failed to generate token: {:?}", result);
        let token_data = result.unwrap();
        
        assert!(!token_data.token.is_empty(), "Token should not be empty");
        assert!(token_data.iat <= Utc::now().timestamp(), "iat should be valid");
        assert!(token_data.exp > token_data.iat, "exp should be after iat");
        assert!(token_data.exp > Utc::now().timestamp(), "Token should not be expired");

        // Cleanup: Delete the test user
        if let Err(e) = repository.delete_by_email(user_dto.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", user_dto.email, e);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_generate_and_retrieve_token_claims_roundtrip() {
        // Arrange: Create a real user and generate a token
        let db = get_database().await;
        let user_service = UserService::new(&db);
        let repository = UserRepository::new(&db);
        
        // Set up JWT secret
        let jwt_secret = get_unique_jwt_secret("roundtrip");
        let token_service = TokenService::new(jwt_secret);
        
        // Create a unique user
        let user_dto = UserFactory::unique_fake_user_register_dto("token_roundtrip");
        let create_result = user_service.create_user(user_dto.clone()).await;
        assert!(create_result.is_ok(), "Failed to create user for roundtrip test");
        
        let user_option = repository.find_by_email(user_dto.email.clone()).await;
        assert!(user_option.is_some(), "User not found in database");
        let user = user_option.unwrap();

        // Generate token
        let token_result = token_service.generate_token(user.clone());
        assert!(token_result.is_ok(), "Failed to generate token");
        let token_data = token_result.unwrap();

        // Act: Retrieve token claims
        let claims_result = token_service.retrieve_token_claims(&token_data.token);

        // Assert: Verify claims are correct
        assert!(claims_result.is_ok(), "Failed to retrieve token claims: {:?}", claims_result);
        let token_claims = claims_result.unwrap();
        
        assert_eq!(token_claims.claims.sub(), user.id, "Subject should match user ID");
        assert_eq!(token_claims.claims.email(), user.email, "Email should match user email");
        assert_eq!(token_claims.claims.iat(), token_data.iat, "iat should match");
        assert_eq!(token_claims.claims.exp(), token_data.exp, "exp should match");

        // Cleanup: Delete the test user
        if let Err(e) = repository.delete_by_email(user_dto.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", user_dto.email, e);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_retrieve_token_claims_with_invalid_token() {
        // Arrange: Set up token service
        let jwt_secret = get_unique_jwt_secret("invalid");
        let token_service = TokenService::new(jwt_secret);
        
        let invalid_tokens = vec![
            "invalid.token.value",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid_payload.signature",
            "",
            "not.a.jwt",
            "too.many.parts.in.this.token",
        ];

        for invalid_token in invalid_tokens {
            // Act: Try to retrieve claims from invalid token
            let result = token_service.retrieve_token_claims(invalid_token);

            // Assert: Should fail for all invalid tokens
            assert!(result.is_err(), "Should fail for invalid token: {}", invalid_token);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_retrieve_token_claims_with_wrong_secret() {
        // Arrange: Create a token with one secret, try to decode with another
        let db = get_database().await;
        let user_service = UserService::new(&db);
        let repository = UserRepository::new(&db);
        
        // Create user
        let user_dto = UserFactory::unique_fake_user_register_dto("wrong_secret");
        let create_result = user_service.create_user(user_dto.clone()).await;
        assert!(
            create_result.is_ok(),
            "Failed to create user for verification test: {:?}",
            create_result.unwrap_err()
        );
        
        let user_option = repository.find_by_email(user_dto.email.clone()).await;
        assert!(user_option.is_some(), "User not found in database");
        let user = user_option.unwrap();

        // Generate token with first secret
        let jwt_secret_1 = get_unique_jwt_secret("secret_one");
        let token_service_1 = TokenService::new(jwt_secret_1);
        let token_result = token_service_1.generate_token(user.clone());
        assert!(token_result.is_ok(), "Failed to generate token");
        let token_data = token_result.unwrap();

        // Try to decode with different secret
        let jwt_secret_2 = get_unique_jwt_secret("secret_two");
        let token_service_2 = TokenService::new(jwt_secret_2);

        // Act: Try to retrieve claims with wrong secret
        let result = token_service_2.retrieve_token_claims(&token_data.token);

        // Assert: Should fail due to wrong secret
        assert!(result.is_err(), "Should fail when using wrong secret");

        // Cleanup: Delete the test user
        if let Err(e) = repository.delete_by_email(user_dto.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", user_dto.email, e);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_token_expiration_time() {
        // Arrange: Create a real user
        let db = get_database().await;
        let user_service = UserService::new(&db);
        let repository = UserRepository::new(&db);
        
        let jwt_secret = get_unique_jwt_secret("expiration");
        let token_service = TokenService::new(jwt_secret);
        
        let user_dto = UserFactory::unique_fake_user_register_dto("token_exp");
        let create_result = user_service.create_user(user_dto.clone()).await;
        assert!(create_result.is_ok(), "Failed to create user");
        
        let user_option = repository.find_by_email(user_dto.email.clone()).await;
        assert!(user_option.is_some(), "User not found in database");
        let user = user_option.unwrap();

        let now_before = Utc::now().timestamp();

        // Act: Generate token
        let result = token_service.generate_token(user.clone());

        let now_after = Utc::now().timestamp();

        // Assert: Verify expiration time is correct
        assert!(result.is_ok(), "Failed to generate token");
        let token_data = result.unwrap();
        
        // Token should expire in approximately 30 minutes (1800 seconds)
        let expected_exp_min = now_before + (token_service.expiration * 60) - 10; // Allow 10 seconds tolerance
        let expected_exp_max = now_after + (token_service.expiration * 60) + 10;
        
        assert!(
            token_data.exp >= expected_exp_min && token_data.exp <= expected_exp_max,
            "Token expiration time should be approximately {} minutes from now. Expected between {} and {}, got {}",
            token_service.expiration,
            expected_exp_min,
            expected_exp_max,
            token_data.exp
        );

        // Cleanup: Delete the test user
        if let Err(e) = repository.delete_by_email(user_dto.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", user_dto.email, e);
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_token_uniqueness() {
        // Arrange: Create multiple users
        let db = get_database().await;
        let user_service = UserService::new(&db);
        let repository = UserRepository::new(&db);
        
        let jwt_secret = get_unique_jwt_secret("uniqueness");
        let token_service = TokenService::new(jwt_secret);
        
        // Create first user
        let user_dto_1 = UserFactory::unique_fake_user_register_dto("unique1");
        let create_result_1 = user_service.create_user(user_dto_1.clone()).await;
        assert!(create_result_1.is_ok(), "Failed to create first user");
        
        let user_option_1 = repository.find_by_email(user_dto_1.email.clone()).await;
        assert!(user_option_1.is_some(), "First user not found");
        let user_1 = user_option_1.unwrap();

        // Create second user
        let user_dto_2 = UserFactory::unique_fake_user_register_dto("unique2");
        let create_result_2 = user_service.create_user(user_dto_2.clone()).await;
        assert!(create_result_2.is_ok(), "Failed to create second user");
        
        let user_option_2 = repository.find_by_email(user_dto_2.email.clone()).await;
        assert!(user_option_2.is_some(), "Second user not found");
        let user_2 = user_option_2.unwrap();

        // Act: Generate tokens for both users
        let token_result_1 = token_service.generate_token(user_1.clone());
        let token_result_2 = token_service.generate_token(user_2.clone());

        // Assert: Tokens should be different
        assert!(token_result_1.is_ok() && token_result_2.is_ok(), "Both tokens should be generated successfully");
        let token_1 = token_result_1.unwrap();
        let token_2 = token_result_2.unwrap();
        
        assert_ne!(token_1.token, token_2.token, "Tokens for different users should be different");

        // Verify each token contains correct user information
        let claims_1 = token_service.retrieve_token_claims(&token_1.token).unwrap();
        let claims_2 = token_service.retrieve_token_claims(&token_2.token).unwrap();
        
        assert_eq!(claims_1.claims.sub(), user_1.id);
        assert_eq!(claims_1.claims.email(), user_1.email);
        assert_eq!(claims_2.claims.sub(), user_2.id);
        assert_eq!(claims_2.claims.email(), user_2.email);

        // Cleanup: Delete both test users
        if let Err(e) = repository.delete_by_email(user_dto_1.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", user_dto_1.email, e);
        }
        if let Err(e) = repository.delete_by_email(user_dto_2.email.clone()).await {
            eprintln!("Cleanup failed for {}: {:?}", user_dto_2.email, e);
        }
    }
}