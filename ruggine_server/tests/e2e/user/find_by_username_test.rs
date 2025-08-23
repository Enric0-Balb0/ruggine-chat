use ruggine_server::dto::user_dto::UserRegisterDto;
use ruggine_server::entity::user::{UserStatus, UserType};
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use serde_json::Value;
use tower::ServiceExt;

use crate::common::{
    cleanup_user_by_email, create_test_user, get_database, login_and_get_token_for_user,
    start_test_server,
};

#[cfg(test)]
mod find_by_username_e2e_tests {
    use super::*;

    /// Helper function to create a test user with a specific username
    async fn create_test_user_with_username(prefix: &str, username: &str) -> (UserRegisterDto, String) {
        let db = get_database().await;
        let user_service = UserService::new(&db);

        let mut user_dto = UserFactory::unique_fake_user_register_dto(prefix);
        user_dto.username = username.to_string();
        let original_password = user_dto.password.clone();

        let create_result = user_service.create_user(user_dto.clone()).await;
        assert!(create_result.is_ok(), "Failed to create user for e2e test");

        (user_dto, original_password)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_success_with_valid_user() {
        // Arrange: Start server and create test user
        let (addr, _shutdown_tx) = start_test_server().await;
        let unique_username = format!("e2e_success_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let (user_dto, original_password) = create_test_user_with_username("e2e_success", &unique_username).await;

        // Get authentication token
        let (_, _, token) = {
            let db = get_database().await;
            let repository = UserRepository::new(&db);
            let user_option = repository.find_by_email(user_dto.email.clone()).await;
            assert!(user_option.is_some(), "User should exist in database");
            let user = user_option.unwrap();
            let token = login_and_get_token_for_user(&user, &original_password).await;
            (user, original_password, token)
        };

        // Act: Send GET request to /api/user/username/{username}
        let client = reqwest::Client::new();
        let response = client
            .get(&format!(
                "http://{}/api/user/username/{}",
                addr, unique_username
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        // Assert: Verify successful response
        assert_eq!(response.status(), 200);

        let response_text = response.text().await.unwrap();
        let response_json: Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = response_json["data"].as_object().unwrap();

        // Verify user data matches
        assert_eq!(data["username"], unique_username);
        assert_eq!(data["email"], user_dto.email);
        assert_eq!(data["first_name"], user_dto.first_name);
        assert_eq!(data["last_name"], user_dto.last_name);
        assert_eq!(data["user_status"], UserStatus::Active.to_string());
        assert_eq!(data["user_type"], UserType::EndUser.to_string());
        assert_eq!(data["birthday"], user_dto.birthday.format("%Y-%m-%d").to_string());
        assert_eq!(data["address"], user_dto.address);
        assert_eq!(data["gender"], user_dto.gender.to_string());
        assert_eq!(data["is_online"], false);

        // Verify password is not included
        assert!(data.get("password").is_none(), "Password should not be in response");

        // Cleanup
        cleanup_user_by_email(user_dto.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_not_found() {
        // Arrange: Start server
        let (addr, _shutdown_tx) = start_test_server().await;
        let non_existent_username = format!("nonexistent_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

        // Create a test user for authentication
        let (auth_user, auth_password) = create_test_user("e2e_auth").await;
        let token = login_and_get_token_for_user(&auth_user, &auth_password).await;

        // Act: Send GET request with non-existent username
        let client = reqwest::Client::new();
        let response = client
            .get(&format!(
                "http://{}/api/user/username/{}",
                addr, non_existent_username
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        // Assert: Verify response for non-existent user
        assert_eq!(response.status(), 200);

        let response_text = response.text().await.unwrap();
        let response_json: Value = serde_json::from_str(&response_text).unwrap();

        // Verify data is null for non-existent user
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        assert!(response_json["data"].is_null(), "Data should be null for non-existent user");

        // Cleanup
        cleanup_user_by_email(auth_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_case_sensitivity() {
        // Arrange: Start server and create test user
        let (addr, _shutdown_tx) = start_test_server().await;
        let original_username = format!("CaseSensitive_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let (user_dto, original_password) = create_test_user_with_username("e2e_case", &original_username).await;

        // Get authentication token
        let (_, _, token) = {
            let db = get_database().await;
            let repository = UserRepository::new(&db);
            let user_option = repository.find_by_email(user_dto.email.clone()).await;
            assert!(user_option.is_some(), "User should exist in database");
            let user = user_option.unwrap();
            let token = login_and_get_token_for_user(&user, &original_password).await;
            (user, original_password, token)
        };

        // Act: Test with exact case
        let client = reqwest::Client::new();
        let response_exact = client
            .get(&format!(
                "http://{}/api/user/username/{}",
                addr, original_username
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        // Assert: Exact case should work
        assert_eq!(response_exact.status(), 200);
        let response_text = response_exact.text().await.unwrap();
        let response_json: Value = serde_json::from_str(&response_text).unwrap();
        assert!(!response_json["data"].is_null(), "Should find user with exact case");

        // Act: Test with different case
        let lowercase_username = original_username.to_lowercase();
        let response_lower = client
            .get(&format!(
                "http://{}/api/user/username/{}",
                addr, lowercase_username
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        // Assert: Different case should not find user (case sensitive)
        assert_eq!(response_lower.status(), 200);
        let response_text = response_lower.text().await.unwrap();
        let response_json: Value = serde_json::from_str(&response_text).unwrap();
        assert!(response_json["data"].is_null(), "Should not find user with different case");

        // Cleanup
        cleanup_user_by_email(user_dto.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_unauthorized_without_token() {
        // Arrange: Start server and create test user
        let (addr, _shutdown_tx) = start_test_server().await;
        let unique_username = format!("e2e_unauth_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let (user_dto, _) = create_test_user_with_username("e2e_unauth", &unique_username).await;

        // Act: Send GET request without authorization token
        let client = reqwest::Client::new();
        let response = client
            .get(&format!(
                "http://{}/api/user/username/{}",
                addr, unique_username
            ))
            .send()
            .await
            .unwrap();

        // Assert: Should be unauthorized
        assert_eq!(response.status(), 401);

        // Cleanup
        cleanup_user_by_email(user_dto.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_invalid_token() {
        // Arrange: Start server and create test user
        let (addr, _shutdown_tx) = start_test_server().await;
        let unique_username = format!("e2e_invalid_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let (user_dto, _) = create_test_user_with_username("e2e_invalid", &unique_username).await;

        // Act: Send GET request with invalid token
        let client = reqwest::Client::new();
        let response = client
            .get(&format!(
                "http://{}/api/user/username/{}",
                addr, unique_username
            ))
            .header("Authorization", "Bearer invalid_token_here")
            .send()
            .await
            .unwrap();

        // Assert: Should be unauthorized
        assert_eq!(response.status(), 401);

        // Cleanup
        cleanup_user_by_email(user_dto.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_malformed_token() {
        // Arrange: Start server and create test user
        let (addr, _shutdown_tx) = start_test_server().await;
        let unique_username = format!("e2e_malformed_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let (user_dto, _) = create_test_user_with_username("e2e_malformed", &unique_username).await;

        // Act: Send GET request with malformed authorization header
        let client = reqwest::Client::new();
        let response = client
            .get(&format!(
                "http://{}/api/user/username/{}",
                addr, unique_username
            ))
            .header("Authorization", "InvalidFormat")
            .send()
            .await
            .unwrap();

        // Assert: Should be unauthorized
        assert_eq!(response.status(), 401);

        // Cleanup
        cleanup_user_by_email(user_dto.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_response_format_validation() {
        // Arrange: Start server and create test user
        let (addr, _shutdown_tx) = start_test_server().await;
        let unique_username = format!("e2e_format_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let (user_dto, original_password) = create_test_user_with_username("e2e_format", &unique_username).await;

        // Get authentication token
        let (_, _, token) = {
            let db = get_database().await;
            let repository = UserRepository::new(&db);
            let user_option = repository.find_by_email(user_dto.email.clone()).await;
            assert!(user_option.is_some(), "User should exist in database");
            let user = user_option.unwrap();
            let token = login_and_get_token_for_user(&user, &original_password).await;
            (user, original_password, token)
        };

        // Act: Send GET request
        let client = reqwest::Client::new();
        let response = client
            .get(&format!(
                "http://{}/api/user/username/{}",
                addr, unique_username
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        // Assert: Verify response format structure
        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        let response_text = response.text().await.unwrap();
        let response_json: Value = serde_json::from_str(&response_text).unwrap();

        // Verify top-level structure
        assert!(response_json.is_object(), "Response should be a JSON object");
        assert!(response_json.get("data").is_some(), "Response should have 'data' field");

        // Verify data structure when user exists
        let data = &response_json["data"];
        assert!(data.is_object(), "Data should be an object when user exists");

        // Verify all required fields are present
        let required_fields = [
            "id", "username", "email", "first_name", "last_name",
            "user_status", "user_type", "birthday", "address", "gender",
            "is_online", "created_at", "updated_at"
        ];

        for field in required_fields.iter() {
            assert!(
                data.get(field).is_some(),
                "Field '{}' should be present in user data",
                field
            );
        }

        // Verify sensitive fields are not present
        let forbidden_fields = ["password", "password_hash"];
        for field in forbidden_fields.iter() {
            assert!(
                data.get(field).is_none(),
                "Field '{}' should not be present in user data",
                field
            );
        }

        // Cleanup
        cleanup_user_by_email(user_dto.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_multiple_users_distinct_results() {
        // Arrange: Start server and create multiple test users
        let (addr, _shutdown_tx) = start_test_server().await;
        let user1_username = format!("e2e_multi1_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let user2_username = format!("e2e_multi2_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

        let (user1_dto, user1_password) = create_test_user_with_username("e2e_multi1", &user1_username).await;
        let (user2_dto, _) = create_test_user_with_username("e2e_multi2", &user2_username).await;

        // Get authentication token using first user
        let (_, _, token) = {
            let db = get_database().await;
            let repository = UserRepository::new(&db);
            let user_option = repository.find_by_email(user1_dto.email.clone()).await;
            assert!(user_option.is_some(), "User should exist in database");
            let user = user_option.unwrap();
            let token = login_and_get_token_for_user(&user, &user1_password).await;
            (user, user1_password, token)
        };

        let client = reqwest::Client::new();

        // Act: Search for first user
        let response1 = client
            .get(&format!(
                "http://{}/api/user/username/{}",
                addr, user1_username
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        // Assert: First user found correctly
        assert_eq!(response1.status(), 200);
        let response1_text = response1.text().await.unwrap();
        let response1_json: Value = serde_json::from_str(&response1_text).unwrap();
        assert!(!response1_json["data"].is_null());
        assert_eq!(response1_json["data"]["username"], user1_username);
        assert_eq!(response1_json["data"]["email"], user1_dto.email);

        // Act: Search for second user
        let response2 = client
            .get(&format!(
                "http://{}/api/user/username/{}",
                addr, user2_username
            ))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        // Assert: Second user found correctly
        assert_eq!(response2.status(), 200);
        let response2_text = response2.text().await.unwrap();
        let response2_json: Value = serde_json::from_str(&response2_text).unwrap();
        assert!(!response2_json["data"].is_null());
        assert_eq!(response2_json["data"]["username"], user2_username);
        assert_eq!(response2_json["data"]["email"], user2_dto.email);

        // Assert: Results are distinct
        assert_ne!(
            response1_json["data"]["id"],
            response2_json["data"]["id"],
            "Different users should have different IDs"
        );
        assert_ne!(
            response1_json["data"]["email"],
            response2_json["data"]["email"],
            "Different users should have different emails"
        );

        // Cleanup
        cleanup_user_by_email(user1_dto.email).await;
        cleanup_user_by_email(user2_dto.email).await;
    }
}
