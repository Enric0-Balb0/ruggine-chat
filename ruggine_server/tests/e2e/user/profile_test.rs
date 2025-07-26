use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use tower::ServiceExt;
use ruggine_server::routes::{auth, user};
use ruggine_server::state::{auth_state::AuthState, token_state::TokenState, user_state::UserState};
use ruggine_server::dto::user_dto::UserRegisterDto;
use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::config::database::DatabaseTrait;
use axum::body::to_bytes;
use crate::common::{cleanup_user, create_user_router, create_auth_router};

#[cfg(test)]
mod profile_e2e_tests {
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

    /// Helper function to login and get a JWT token
    async fn login_and_get_token(user_dto: &UserRegisterDto, password: &str) -> String {
        let auth_app = create_auth_router().await;
        
        let login_payload = json!({
            "email": user_dto.email,
            "password": password
        });

        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(login_payload.to_string()))
            .unwrap();

        let response = auth_app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK, "Login should succeed");

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure contains user data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = &response_json["data"];

        data["token"].as_str().unwrap().to_string()
    }

    #[tokio::test]
    async fn test_profile_success_with_valid_token() {
        // Arrange: Create router, user, and get token
        let app = create_user_router().await;
        let (user_dto, password) = create_test_user("e2e_profile_success").await;
        let token = login_and_get_token(&user_dto, &password).await;

        // Act: Send GET request to /profile with valid token
        let request = Request::builder()
            .method("GET")
            .uri("/profile")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with user data
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure contains user data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = &response_json["data"];
        
        assert!(data.get("id").is_some(), "User data should contain id");
        assert!(data.get("email").is_some(), "User data should contain email");
        assert!(data.get("username").is_some(), "User data should contain username");
        assert!(data.get("first_name").is_some(), "User data should contain first_name");
        assert!(data.get("last_name").is_some(), "User data should contain last_name");
        assert!(data.get("created_at").is_some(), "User data should contain created_at");
        assert!(data.get("is_active").is_some(), "User data should contain is_active");

        // Verify user data matches expected values
        assert_eq!(data["email"], user_dto.email);
        assert_eq!(data["username"], user_dto.username);
        assert_eq!(data["first_name"], user_dto.first_name);
        assert_eq!(data["last_name"], user_dto.last_name);
        assert_eq!(data["is_active"], 1);

        // Verify password is not included in response
        assert!(data.get("password").is_none(), "User data should not contain password");

        // Cleanup
        cleanup_user(user_dto.email).await;
    }

    #[tokio::test]
    async fn test_profile_failure_without_token() {
        // Arrange: Create router only (no token)
        let app = create_user_router().await;

        // Act: Send GET request to /profile without authorization header
        let request = Request::builder()
            .method("GET")
            .uri("/profile")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized (missing token)
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify error response structure
        assert!(response_json.get("message").is_some(), "Error response should contain message");
        assert!(response_json.get("code").is_some(), "Error response should contain code");
        assert_eq!(response_json["code"], 401);
    }

    #[tokio::test]
    async fn test_profile_failure_with_invalid_token() {
        // Arrange: Create router
        let app = create_user_router().await;

        // Act: Send GET request to /profile with invalid token
        let request = Request::builder()
            .method("GET")
            .uri("/profile")
            .header("authorization", "Bearer invalid.jwt.token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized (invalid token)
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify error response structure
        assert!(response_json.get("message").is_some(), "Error response should contain message");
        assert!(response_json.get("code").is_some(), "Error response should contain code");
        assert_eq!(response_json["code"], 401);
    }

    #[tokio::test]
    async fn test_profile_failure_with_malformed_token() {
        // Arrange: Create router
        let app = create_user_router().await;

        // Act: Send GET request to /profile with malformed authorization header
        let request = Request::builder()
            .method("GET")
            .uri("/profile")
            .header("authorization", "InvalidHeaderFormat")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized (malformed token)
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify error response structure
        assert!(response_json.get("message").is_some(), "Error response should contain message");
        assert!(response_json.get("code").is_some(), "Error response should contain code");
        assert_eq!(response_json["code"], 401);
    }

    #[tokio::test]
    async fn test_profile_failure_with_empty_bearer_token() {
        // Arrange: Create router
        let app = create_user_router().await;

        // Act: Send GET request to /profile with empty bearer token
        let request = Request::builder()
            .method("GET")
            .uri("/profile")
            .header("authorization", "Bearer ")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized (empty token)
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify error response structure
        assert!(response_json.get("message").is_some(), "Error response should contain message");
        assert!(response_json.get("code").is_some(), "Error response should contain code");
        assert_eq!(response_json["code"], 401);
    }

    #[tokio::test]
    async fn test_profile_failure_with_inactive_user() {
        // Arrange: Create router, user, get token, then deactivate user
        let app = create_user_router().await;
        let (user_dto, password) = create_test_user("e2e_profile_inactive").await;
        let token = login_and_get_token(&user_dto, &password).await;

        // Deactivate the user after getting the token
        let db = get_database().await;
        let pool = db.get_pool();
        let update_result = sqlx::query("UPDATE user SET is_active = 0 WHERE email = ?")
            .bind(&user_dto.email)
            .execute(pool)
            .await;
        assert!(update_result.is_ok(), "Failed to deactivate user");

        // Act: Send GET request to /profile with token of inactive user
        let request = Request::builder()
            .method("GET")
            .uri("/profile")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
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
        cleanup_user(user_dto.email).await;
    }

    #[tokio::test]
    async fn test_profile_wrong_http_method() {
        // Arrange: Create router, user, and get token
        let app = create_user_router().await;
        let (user_dto, password) = create_test_user("e2e_profile_wrong_method").await;
        let token = login_and_get_token(&user_dto, &password).await;

        // Act: Send POST request to /profile (should be GET)
        let request = Request::builder()
            .method("POST")
            .uri("/profile")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 405 Method Not Allowed
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);

        // Cleanup
        cleanup_user(user_dto.email).await;
    }

    #[tokio::test]
    async fn test_profile_with_updated_user_data() {
        // Arrange: Create router, user, get token, and update user data
        let app = create_user_router().await;
        let (user_dto, password) = create_test_user("e2e_profile_updated").await;
        let token = login_and_get_token(&user_dto, &password).await;

        // Update user data in database
        let db = get_database().await;
        let pool = db.get_pool();
        let new_first_name = "UpdatedFirstName";
        let new_last_name = "UpdatedLastName";
        let update_time = chrono::Utc::now();

        let update_result = sqlx::query(
            "UPDATE user SET first_name = ?, last_name = ?, updated_at = ? WHERE email = ?"
        )
        .bind(new_first_name)
        .bind(new_last_name)
        .bind(update_time)
        .bind(&user_dto.email)
        .execute(pool)
        .await;
        assert!(update_result.is_ok(), "Failed to update user");

        // Act: Send GET request to /profile with valid token
        let request = Request::builder()
            .method("GET")
            .uri("/profile")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with updated user data
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = &response_json["data"];
        
        // Verify updated data is returned
        assert_eq!(data["first_name"], new_first_name);
        assert_eq!(data["last_name"], new_last_name);
        assert_eq!(data["email"], user_dto.email);
        assert!(data.get("updated_at").is_some(), "Updated timestamp should be present");

        // Cleanup
        cleanup_user(user_dto.email).await;
    }

    #[tokio::test]
    async fn test_profile_response_structure_consistency() {
        // Arrange: Create router, user, and get token
        let app = create_user_router().await;
        let (user_dto, password) = create_test_user("e2e_profile_structure").await;
        let token = login_and_get_token(&user_dto, &password).await;

        // Act: Send GET request to /profile
        let request = Request::builder()
            .method("GET")
            .uri("/profile")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Verify response structure is consistent and complete
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = &response_json["data"];

        // Verify all expected fields are present and have correct types
        assert!(data["id"].is_number(), "id should be a number");
        assert!(data["email"].is_string(), "email should be a string");
        assert!(data["username"].is_string(), "username should be a string");
        assert!(data["first_name"].is_string(), "first_name should be a string");
        assert!(data["last_name"].is_string(), "last_name should be a string");
        assert!(data["created_at"].is_string(), "created_at should be a string");
        assert!(data["is_active"].is_number(), "is_active should be a number");

        // Verify field values are reasonable
        assert!(data["id"].as_i64().unwrap() > 0, "id should be positive");
        assert!(data["email"].as_str().unwrap().contains('@'), "email should be valid format");
        assert!(!data["username"].as_str().unwrap().is_empty(), "username should not be empty");
        assert!(!data["first_name"].as_str().unwrap().is_empty(), "first_name should not be empty");
        assert!(!data["last_name"].as_str().unwrap().is_empty(), "last_name should not be empty");
        assert!(data["is_active"].as_i64().unwrap() >= 0, "is_active should be 0 or 1");

        // Cleanup
        cleanup_user(user_dto.email).await;
    }

    #[tokio::test]
    async fn test_profile_concurrent_requests() {
        // Arrange: Create router, user, and get token
        let app = create_user_router().await;
        let (user_dto, password) = create_test_user("e2e_profile_concurrent_requests").await;
        let token = login_and_get_token(&user_dto, &password).await;

        // Act: Send multiple concurrent requests to /profile
        let token_clone = token.clone();
        let (response1, response2) = tokio::join!(
            async {
                let request = Request::builder()
                    .method("GET")
                    .uri("/profile")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap();
                create_user_router().await.oneshot(request).await.unwrap()
            },
            async {
                let request = Request::builder()
                    .method("GET")
                    .uri("/profile")
                    .header("authorization", format!("Bearer {}", token_clone))
                    .body(Body::empty())
                    .unwrap();
                create_user_router().await.oneshot(request).await.unwrap()
            }
        );

        // Assert: Both requests should succeed
        assert_eq!(response1.status(), StatusCode::OK);
        assert_eq!(response2.status(), StatusCode::OK);

        // Verify both responses contain the same user data
        let body1 = to_bytes(response1.into_body(), usize::MAX).await.unwrap();
        let body2 = to_bytes(response2.into_body(), usize::MAX).await.unwrap();

        let response1_json: serde_json::Value = serde_json::from_str(&String::from_utf8(body1.to_vec()).unwrap()).unwrap();
        let response2_json: serde_json::Value = serde_json::from_str(&String::from_utf8(body2.to_vec()).unwrap()).unwrap();

        assert_eq!(response1_json["data"]["id"], response2_json["data"]["id"]);
        assert_eq!(response1_json["data"]["email"], response2_json["data"]["email"]);

        // Cleanup
        cleanup_user(user_dto.email).await;
    }
}