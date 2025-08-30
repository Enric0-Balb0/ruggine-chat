use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use ruggine_server::dto::user_dto::UserRegisterDto;
use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::config::database::DatabaseTrait;
use crate::common::{cleanup_user_by_email, create_auth_router};
use axum::body::to_bytes;

#[cfg(test)]
mod login_e2e_tests {
    use ruggine_server::entity::user::UserStatus;

    use crate::get_database;
    use super::*;

    /// Helper function to create a real user in the database
    async fn create_test_user(prefix: &str) -> (UserRegisterDto, String) {
        let db = get_database().await;
        let user_service = UserService::new(&db);
        
        let user_dto = UserFactory::unique_fake_user_register_dto(prefix);
        let original_password = user_dto.password.clone();
        
        let create_result = user_service.create_user(user_dto.clone()).await;
        assert!(create_result.is_ok(), "Failed to create user for e2e test");
        
        (user_dto, original_password)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_login_success_with_valid_credentials() {
        // Arrange: Create router and test user
        let app = create_auth_router().await;
        let (user_dto, original_password) = create_test_user("e2e_login_success").await;

        let login_payload = json!({
            "email": user_dto.email,
            "password": original_password
        });

        // Act: Send POST request to /login
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(login_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with token
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure contains user data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = &response_json["data"];

        // Verify response structure
        assert!(data.get("token").is_some(), "Response should contain token");
        assert!(data.get("iat").is_some(), "Response should contain iat");
        assert!(data.get("exp").is_some(), "Response should contain exp");

        let token = data["token"].as_str().unwrap();
        assert!(!token.is_empty(), "Token should not be empty");
        assert!(token.len() > 50, "Token should be substantial length");

        // Cleanup
        cleanup_user_by_email(user_dto.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_login_failure_with_invalid_email() {
        // Arrange: Create router (no user needed)
        let app = create_auth_router().await;

        let login_payload = json!({
            "email": "nonexistent@example.com",
            "password": "any_password"
        });

        // Act: Send POST request to /login with non-existent email
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(login_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found (user not found)
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify error response structure
        assert!(response_json.get("message").is_some(), "Error response should contain message");
        assert!(response_json.get("code").is_some(), "Error response should contain code");
        assert_eq!(response_json["code"], 404);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_login_failure_with_wrong_password() {
        // Arrange: Create router and test user
        let app = create_auth_router().await;
        let (user_dto, _) = create_test_user("e2e_wrong_password").await;

        let login_payload = json!({
            "email": user_dto.email,
            "password": "wrong_password"
        });

        // Act: Send POST request to /login with wrong password
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(login_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized (invalid password) or 400 Bad Request (validation error)
        assert!(
            response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::BAD_REQUEST,
            "Should return 401 Unauthorized or 400 Bad Request for wrong password, got {}",
            response.status()
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify error response structure
        assert!(response_json.get("message").is_some(), "Error response should contain message");
        assert!(response_json.get("code").is_some(), "Error response should contain code");
        assert_eq!(response_json["code"], 401);

        // Cleanup
        cleanup_user_by_email(user_dto.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_login_failure_with_inactive_user() {
        // Arrange: Create router and test user, then deactivate
        let app = create_auth_router().await;
        let (user_dto, original_password) = create_test_user("e2e_inactive_user").await;

        // Deactivate the user directly in database
        let db = get_database().await;
        let pool = db.get_pool();
        let update_result = sqlx::query(r#"UPDATE "user" SET user_status = $1 WHERE email = $2"#)
            .bind(UserStatus::Deleted)
            .bind(&user_dto.email)
            .execute(pool)
            .await;
        assert!(update_result.is_ok(), "Failed to deactivate user");

        let login_payload = json!({
            "email": user_dto.email,
            "password": original_password
        });

        // Act: Send POST request to /login with inactive user
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(login_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 403 Forbidden (user not active)
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify error response structure
        assert!(response_json.get("message").is_some(), "Error response should contain message");
        assert!(response_json.get("code").is_some(), "Error response should contain code");
        assert_eq!(response_json["code"], 403);

        // Cleanup
        cleanup_user_by_email(user_dto.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_login_failure_with_malformed_json() {
        // Arrange: Create router
        let app = create_auth_router().await;

        let malformed_payload = r#"{"email": "test@example.com", "password": "missing_closing_brace""#;

        // Act: Send POST request to /login with malformed JSON
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(malformed_payload))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request (malformed JSON)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_login_failure_with_missing_fields() {
        // Arrange: Create router
        let app = create_auth_router().await;

        let incomplete_payload = json!({
            "email": "test@example.com"
            // Missing password field
        });

        // Act: Send POST request to /login with missing password
        let request = Request::builder()
            .method("POST")
            .uri("/login")
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

    #[tokio_shared_rt::test(shared)]
    async fn test_login_failure_with_empty_fields() {
        // Arrange: Create router
        let app = create_auth_router().await;

        let empty_payload = json!({
            "email": "",
            "password": ""
        });

        // Act: Send POST request to /login with empty fields
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(empty_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request or 404 Not Found
        assert!(
            response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND,
            "Should return 400 or 404 for empty fields"
        );
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_login_wrong_http_method() {
        // Arrange: Create router
        let app = create_auth_router().await;

        let login_payload = json!({
            "email": "test@example.com",
            "password": "password123"
        });

        // Act: Send GET request to /login (should be POST)
        let request = Request::builder()
            .method("GET")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(login_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 405 Method Not Allowed
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_login_missing_content_type() {
        // Arrange: Create router and test user
        let app = create_auth_router().await;
        let (user_dto, original_password) = create_test_user("e2e_no_content_type").await;

        let login_payload = json!({
            "email": user_dto.email,
            "password": original_password
        });

        // Act: Send POST request to /login without content-type header
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            // Missing content-type header
            .body(Body::from(login_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return an error (400 or 415)
        assert!(
            response.status() == StatusCode::BAD_REQUEST 
            || response.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE
            || response.status() == StatusCode::OK, // Some frameworks are lenient
            "Should return error for missing content-type or be lenient"
        );

        // Cleanup
        cleanup_user_by_email(user_dto.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_login_token_validity_and_structure() {
        // Arrange: Create router and test user
        let app = create_auth_router().await;
        let (user_dto, original_password) = create_test_user("e2e_token_validity").await;

        let login_payload = json!({
            "email": user_dto.email,
            "password": original_password
        });

        // Act: Send POST request to /login
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(login_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Verify token structure and validity
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure contains user data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = &response_json["data"];

        // Verify token structure
        let token = data["token"].as_str().unwrap();
        let iat = data["iat"].as_i64().unwrap();
        let exp = data["exp"].as_i64().unwrap();

        // Token should be a JWT (3 parts separated by dots)
        let token_parts: Vec<&str> = token.split('.').collect();
        assert_eq!(token_parts.len(), 3, "JWT should have 3 parts");

        // Timestamps should be reasonable
        assert!(iat > 0, "iat should be positive");
        assert!(exp > iat, "exp should be greater than iat");
        assert!(exp > chrono::Utc::now().timestamp(), "Token should not be expired");

        // Cleanup
        cleanup_user_by_email(user_dto.email).await;
    }
}