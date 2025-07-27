use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use tower::ServiceExt;
use ruggine_server::routes::user;
use ruggine_server::state::{token_state::TokenState, user_state::UserState};
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::config::database::DatabaseTrait;
use axum::body::to_bytes;
use crate::common::{cleanup_user, create_user_router};

#[cfg(test)]
mod register_e2e_tests {
    use ruggine_server::entity::user::UserStatus;

    use crate::get_database;
    use super::*;

    #[tokio::test]
    async fn test_register_success_with_valid_data() {
        // Arrange: Create router and registration data
        let app = create_user_router().await;
        let register_dto = UserFactory::unique_fake_user_register_dto("e2e_register_success");

        let register_payload = json!({
            "email": register_dto.email,
            "password": register_dto.password,
            "username": register_dto.username,
            "first_name": register_dto.first_name,
            "last_name": register_dto.last_name
        });

        // Act: Send POST request to /register
        let request = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(register_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with user data
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json.get("data").expect("Missing 'data' in response");

        // Verify response contains user data (not wrapped in data field for register)
        assert!(data.get("id").is_some(), "Response should contain id");
        assert!(data.get("email").is_some(), "Response should contain email");
        assert!(data.get("username").is_some(), "Response should contain username");
        assert!(data.get("first_name").is_some(), "Response should contain first_name");
        assert!(data.get("last_name").is_some(), "Response should contain last_name");
        assert!(data.get("created_at").is_some(), "Response should contain created_at");
        assert!(data.get("updated_at").is_some(), "Response should contain updated_at");
        assert!(data.get("user_status").is_some(), "Response should contain user_status");

        // Verify user data matches input
        assert_eq!(data["email"], register_dto.email);
        assert_eq!(data["username"], register_dto.username);
        assert_eq!(data["first_name"], register_dto.first_name);
        assert_eq!(data["last_name"], register_dto.last_name);
        assert_eq!(data["user_status"], UserStatus::Active.to_string());

        // Verify password is not included in response
        assert!(data.get("password").is_none(), "Response should not contain password");

        // Verify user ID is positive
        assert!(data["id"].as_i64().unwrap() > 0, "User ID should be positive");

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_creates_user_in_database() {
        // Arrange: Create router and registration data
        let app = create_user_router().await;
        let register_dto = UserFactory::unique_fake_user_register_dto("e2e_register_db_check");

        let register_payload = json!({
            "email": register_dto.email,
            "password": register_dto.password,
            "username": register_dto.username,
            "first_name": register_dto.first_name,
            "last_name": register_dto.last_name
        });

        // Act: Send POST request to /register
        let request = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(register_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Assert: Verify user exists in database
        let db = get_database().await;
        let repository = UserRepository::new(&db);
        let stored_user = repository.find_by_email(register_dto.email.clone()).await;

        assert!(stored_user.is_some(), "User should exist in database");
        let stored_user = stored_user.unwrap();

        assert_eq!(stored_user.email, register_dto.email);
        assert_eq!(stored_user.username, register_dto.username);
        assert_eq!(stored_user.first_name, register_dto.first_name);
        assert_eq!(stored_user.last_name, register_dto.last_name);
        assert_eq!(stored_user.user_status, UserStatus::Active);

        // Verify password is hashed
        assert_ne!(stored_user.password, register_dto.password, "Password should be hashed");

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_failure_with_duplicate_email() {
        // Arrange: Create router and register a user first
        let app = create_user_router().await;
        let register_dto = UserFactory::unique_fake_user_register_dto("e2e_register_duplicate");

        let register_payload = json!({
            "email": register_dto.email,
            "password": register_dto.password,
            "username": register_dto.username,
            "first_name": register_dto.first_name,
            "last_name": register_dto.last_name
        });

        // Register user first time
        let request1 = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(register_payload.to_string()))
            .unwrap();

        let response1 = create_user_router().await.oneshot(request1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::OK);

        // Act: Try to register the same user again
        let request2 = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(register_payload.to_string()))
            .unwrap();

        let response2 = create_user_router().await.oneshot(request2).await.unwrap();

        // Assert: Should return 409 Conflict (user already exists)
        assert_eq!(response2.status(), StatusCode::CONFLICT);

        let body = to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify error response structure
        assert!(response_json.get("message").is_some(), "Error response should contain message");
        assert!(response_json.get("code").is_some(), "Error response should contain code");
        assert_eq!(response_json["code"], 409);

        // Cleanup
        cleanup_user(register_dto.email).await;
    }

    #[tokio::test]
    async fn test_register_failure_with_malformed_json() {
        // Arrange: Create router
        let app = create_user_router().await;

        let malformed_payload = r#"{"email": "test@example.com", "password": "missing_closing_brace""#;

        // Act: Send POST request to /register with malformed JSON
        let request = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(malformed_payload))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request (malformed JSON)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_register_failure_with_missing_required_fields() {
        // Arrange: Create router
        let app = create_user_router().await;

        let incomplete_payload = json!({
            "email": "test@example.com",
            "password": "password123"
            // Missing username, first_name, last_name
        });

        // Act: Send POST request to /register with missing fields
        let request = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(incomplete_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request (validation error)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify error response structure
        assert!(response_json.get("message").is_some(), "Error response should contain message");
        assert!(response_json.get("code").is_some(), "Error response should contain code");
        assert_eq!(response_json["code"], 400);
    }

    #[tokio::test]
    async fn test_register_failure_with_invalid_email_format() {
        // Arrange: Create router
        let app = create_user_router().await;

        let invalid_email_payload = json!({
            "email": "invalid-email-format",
            "password": "password123",
            "username": "testuser",
            "first_name": "Test",
            "last_name": "User"
        });

        // Act: Send POST request to /register with invalid email
        let request = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(invalid_email_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request (validation error)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify error response structure
        assert!(response_json.get("message").is_some(), "Error response should contain message");
        assert!(response_json.get("code").is_some(), "Error response should contain code");
        assert_eq!(response_json["code"], 400);
    }

    #[tokio::test]
    async fn test_register_failure_with_empty_fields() {
        // Arrange: Create router
        let app = create_user_router().await;

        let empty_fields_payload = json!({
            "email": "",
            "password": "",
            "username": "",
            "first_name": "",
            "last_name": ""
        });

        // Act: Send POST request to /register with empty fields
        let request = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(empty_fields_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request (validation error)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify error response structure
        assert!(response_json.get("message").is_some(), "Error response should contain message");
        assert!(response_json.get("code").is_some(), "Error response should contain code");
        assert_eq!(response_json["code"], 400);
    }

    #[tokio::test]
    async fn test_register_wrong_http_method() {
        // Arrange: Create router
        let app = create_user_router().await;

        let register_payload = json!({
            "email": "test@example.com",
            "password": "password123",
            "username": "testuser",
            "first_name": "Test",
            "last_name": "User"
        });

        // Act: Send GET request to /register (should be POST)
        let request = Request::builder()
            .method("GET")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(register_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 405 Method Not Allowed
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_register_with_special_characters() {
        // Arrange: Create router
        let app = create_user_router().await;

        let special_chars_payload = json!({
            "email": "special.chars+test@example.com",
            "password": "P@ssw0rd!#$%",
            "username": "user_with_underscore_123",
            "first_name": "José María",
            "last_name": "García-López"
        });

        // Act: Send POST request to /register with special characters
        let request = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(special_chars_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should succeed with special characters
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json.get("data").expect("Missing 'data' in response");

        // Verify special characters are preserved
        assert_eq!(data["email"], "special.chars+test@example.com");
        assert_eq!(data["username"], "user_with_underscore_123");
        assert_eq!(data["first_name"], "José María");
        assert_eq!(data["last_name"], "García-López");

        // Cleanup
        cleanup_user("special.chars+test@example.com".to_string()).await;
    }

    #[tokio::test]
    async fn test_register_missing_content_type() {
        // Arrange: Create router
        let app = create_user_router().await;
        let register_dto = UserFactory::unique_fake_user_register_dto("e2e_no_content_type");

        let register_payload = json!({
            "email": register_dto.email,
            "password": register_dto.password,
            "username": register_dto.username,
            "first_name": register_dto.first_name,
            "last_name": register_dto.last_name
        });

        // Act: Send POST request to /register without content-type header
        let request = Request::builder()
            .method("POST")
            .uri("/register")
            // Missing content-type header
            .body(Body::from(register_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return an error (400 or 415) or be lenient
        assert!(
            response.status() == StatusCode::BAD_REQUEST 
            || response.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE
            || response.status() == StatusCode::OK, // Some frameworks are lenient
            "Should return error for missing content-type or be lenient"
        );

        // Cleanup if registration succeeded
        if response.status() == StatusCode::OK {
            cleanup_user(register_dto.email).await;
        }
    }

    #[tokio::test]
    async fn test_register_concurrent_different_users() {
        // Arrange: Create router and multiple registration data
        let register_dto1 = UserFactory::unique_fake_user_register_dto("e2e_concurrent1");
        let register_dto2 = UserFactory::unique_fake_user_register_dto("e2e_concurrent2");

        let payload1 = json!({
            "email": register_dto1.email,
            "password": register_dto1.password,
            "username": register_dto1.username,
            "first_name": register_dto1.first_name,
            "last_name": register_dto1.last_name
        });

        let payload2 = json!({
            "email": register_dto2.email,
            "password": register_dto2.password,
            "username": register_dto2.username,
            "first_name": register_dto2.first_name,
            "last_name": register_dto2.last_name
        });

        // Act: Send concurrent registration requests
        let (response1, response2) = tokio::join!(
            async {
                let request = Request::builder()
                    .method("POST")
                    .uri("/register")
                    .header("content-type", "application/json")
                    .body(Body::from(payload1.to_string()))
                    .unwrap();
                create_user_router().await.oneshot(request).await.unwrap()
            },
            async {
                let request = Request::builder()
                    .method("POST")
                    .uri("/register")
                    .header("content-type", "application/json")
                    .body(Body::from(payload2.to_string()))
                    .unwrap();
                create_user_router().await.oneshot(request).await.unwrap()
            }
        );

        // Assert: Both registrations should succeed
        assert_eq!(response1.status(), StatusCode::OK);
        assert_eq!(response2.status(), StatusCode::OK);

        // Verify both users have different IDs
        let body1 = to_bytes(response1.into_body(), usize::MAX).await.unwrap();
        let body2 = to_bytes(response2.into_body(), usize::MAX).await.unwrap();

        let response1_json: serde_json::Value = serde_json::from_str(&String::from_utf8(body1.to_vec()).unwrap()).unwrap();
        let response2_json: serde_json::Value = serde_json::from_str(&String::from_utf8(body2.to_vec()).unwrap()).unwrap();

        let data1 = response1_json.get("data").expect("Missing 'data' in response");
        let data2 = response2_json.get("data").expect("Missing 'data' in response");

        assert_ne!(data1["id"], data2["id"], "Users should have different IDs");
        assert_ne!(data1["email"], data2["email"], "Users should have different emails");

        // Cleanup
        cleanup_user(register_dto1.email).await;
        cleanup_user(register_dto2.email).await;
    }

    #[tokio::test]
    async fn test_register_response_structure_consistency() {
        // Arrange: Create router and registration data
        let app = create_user_router().await;
        let register_dto = UserFactory::unique_fake_user_register_dto("e2e_structure_check");

        let register_payload = json!({
            "email": register_dto.email,
            "password": register_dto.password,
            "username": register_dto.username,
            "first_name": register_dto.first_name,
            "last_name": register_dto.last_name
        });

        // Act: Send POST request to /register
        let request = Request::builder()
            .method("POST")
            .uri("/register")
            .header("content-type", "application/json")
            .body(Body::from(register_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Verify response structure is consistent and complete
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json.get("data").expect("Missing 'data' in response");

        // Verify all expected fields are present and have correct types
        assert!(data["id"].is_number(), "id should be a number");
        assert!(data["email"].is_string(), "email should be a string");
        assert!(data["username"].is_string(), "username should be a string");
        assert!(data["first_name"].is_string(), "first_name should be a string");
        assert!(data["last_name"].is_string(), "last_name should be a string");
        assert!(data["created_at"].is_string(), "created_at should be a string");
        assert!(data["user_status"].is_string(), "user_status should be a string");

        // Verify field values are reasonable
        assert!(data["id"].as_i64().unwrap() > 0, "id should be positive");
        assert!(data["email"].as_str().unwrap().contains('@'), "email should be valid format");
        assert!(!data["username"].as_str().unwrap().is_empty(), "username should not be empty");
        assert!(!data["first_name"].as_str().unwrap().is_empty(), "first_name should not be empty");
        assert!(!data["last_name"].as_str().unwrap().is_empty(), "last_name should not be empty");
        assert_eq!(data["user_status"].as_str().unwrap(), UserStatus::Active.to_string(), "user_status should be active for new users");

        // Verify sensitive fields are not included
        assert!(data.get("password").is_none(), "password should not be in response");

        // Cleanup
        cleanup_user(register_dto.email).await;
    }
}