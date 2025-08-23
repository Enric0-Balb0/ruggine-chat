
#[cfg(test)]
mod find_by_id_handler_e2e_tests {
    use crate::common::{cleanup_user_by_email, create_test_user, login_and_get_token_for_user, start_test_server};
    use serde_json::Value;

    #[tokio_shared_rt::test(shared)]
    async fn test_e2e_find_by_id_existing_user_success() {
        // Arrange: Create test users
        let (target_user, _) = create_test_user("e2e_find_target").await;
        let (auth_user, auth_password) = create_test_user("e2e_find_auth").await;

        // Start test server and get authentication token
        let (addr, _shutdown_tx) = start_test_server().await;
        let token = login_and_get_token_for_user(&auth_user, &auth_password).await;

        // Create request
        let url = format!("http://{}/api/user/{}", addr, target_user.id);
        let client = reqwest::Client::new();

        // Act: Send authenticated request
        let response = client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request");

        // Assert: Check response status and body
        assert_eq!(response.status(), 200, "Should return 200 OK");

        let response_body: Value = response.json().await.expect("Failed to parse JSON");

        // Verify response structure
        assert!(response_body.get("data").is_some(), "Response should have data field");

        let user_data = &response_body["data"];
        assert_eq!(user_data["id"], target_user.id, "User ID should match");
        assert_eq!(user_data["email"], target_user.email, "Email should match");
        assert_eq!(user_data["username"], target_user.username, "Username should match");
        assert_eq!(user_data["first_name"], target_user.first_name, "First name should match");
        assert_eq!(user_data["last_name"], target_user.last_name, "Last name should match");

        // Verify password is not included in response
        assert!(user_data.get("password").is_none(), "Password should not be in response");

        // Cleanup
        cleanup_user_by_email(target_user.email).await;
        cleanup_user_by_email(auth_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_e2e_find_by_id_non_existent_user() {
        // Arrange: Create auth user but no target user
        let (auth_user, auth_password) = create_test_user("e2e_find_auth_nonexistent").await;
        let non_existent_id = 99999;

        // Start test server and get authentication token
        let (addr, _shutdown_tx) = start_test_server().await;
        let token = login_and_get_token_for_user(&auth_user, &auth_password).await;

        // Create request
        let url = format!("http://{}/api/user/{}", addr, non_existent_id);
        let client = reqwest::Client::new();

        // Act: Send authenticated request for non-existent user
        let response = client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request");

        // Assert: Should return 404 Not Found
        assert_eq!(response.status(), 404, "Should return 404 for non-existent user");

        let response_body: Value = response.json().await.expect("Failed to parse JSON");
        assert!(response_body.get("message").is_some(), "Error should have message");

        // Cleanup
        cleanup_user_by_email(auth_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_e2e_find_by_id_unauthorized_no_token() {
        // Arrange: Create target user
        let (target_user, _) = create_test_user("e2e_find_unauthorized").await;

        // Start test server
        let (addr, _shutdown_tx) = start_test_server().await;

        // Create request without authentication token
        let url = format!("http://{}/api/user/{}", addr, target_user.id);
        let client = reqwest::Client::new();

        // Act: Send unauthenticated request
        let response = client
            .get(&url)
            .send()
            .await
            .expect("Failed to send request");

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), 401, "Should return 401 for unauthenticated request");

        // Cleanup
        cleanup_user_by_email(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_e2e_find_by_id_invalid_token() {
        // Arrange: Create target user
        let (target_user, _) = create_test_user("e2e_find_invalid_token").await;

        // Start test server
        let (addr, _shutdown_tx) = start_test_server().await;
        let invalid_token = "invalid.jwt.token";

        // Create request with invalid token
        let url = format!("http://{}/api/user/{}", addr, target_user.id);
        let client = reqwest::Client::new();

        // Act: Send request with invalid token
        let response = client
            .get(&url)
            .bearer_auth(invalid_token)
            .send()
            .await
            .expect("Failed to send request");

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), 401, "Should return 401 for invalid token");

        // Cleanup
        cleanup_user_by_email(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_e2e_find_by_id_user_can_find_self() {
        // Arrange: Create a user
        let (user, password) = create_test_user("e2e_find_self").await;

        // Start test server and get authentication token
        let (addr, _shutdown_tx) = start_test_server().await;
        let token = login_and_get_token_for_user(&user, &password).await;

        // Create request to find own user data
        let url = format!("http://{}/api/user/{}", addr, user.id);
        let client = reqwest::Client::new();

        // Act: Send authenticated request to find self
        let response = client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request");

        // Assert: Should successfully return own user data
        assert_eq!(response.status(), 200, "User should be able to find their own data");

        let response_body: Value = response.json().await.expect("Failed to parse JSON");

        // Verify response structure and data
        let user_data = &response_body["data"];
        assert_eq!(user_data["id"], user.id, "Should return own user ID");
        assert_eq!(user_data["email"], user.email, "Should return own email");
        assert_eq!(user_data["username"], user.username, "Should return own username");

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_e2e_find_by_id_invalid_user_id_format() {
        // Arrange: Create auth user
        let (auth_user, auth_password) = create_test_user("e2e_find_invalid_id").await;

        // Start test server and get authentication token
        let (addr, _shutdown_tx) = start_test_server().await;
        let token = login_and_get_token_for_user(&auth_user, &auth_password).await;

        // Create request with invalid ID format (string instead of integer)
        let url = format!("http://{}/api/user/invalid_id", addr);
        let client = reqwest::Client::new();

        // Act: Send request with invalid ID format
        let response = client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request");

        // Assert: Should return 400 Bad Request or 404 depending on router behavior
        assert!(
            response.status() == 400 || response.status() == 404,
            "Should return 400 or 404 for invalid ID format, got: {}",
            response.status()
        );

        // Cleanup
        cleanup_user_by_email(auth_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_e2e_find_by_id_response_format() {
        // Arrange: Create test users
        let (target_user, _) = create_test_user("e2e_find_response_format").await;
        let (auth_user, auth_password) = create_test_user("e2e_find_auth_format").await;

        // Start test server and get authentication token
        let (addr, _shutdown_tx) = start_test_server().await;
        let token = login_and_get_token_for_user(&auth_user, &auth_password).await;

        // Create request
        let url = format!("http://{}/api/user/{}", addr, target_user.id);
        let client = reqwest::Client::new();

        // Act: Send authenticated request
        let response = client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request");

        // Assert: Verify complete response format
        assert_eq!(response.status(), 200, "Should return 200 OK");

        let response_body: Value = response.json().await.expect("Failed to parse JSON");

        // Check standard API response structure
        assert!(response_body.get("data").is_some(), "Should have data field");

        let user_data = &response_body["data"];

        // Verify all expected UserReadDto fields are present
        let expected_fields = [
            "id", "email", "username", "first_name", "last_name",
            "user_status", "user_type", "birthday", "address", "gender",
            "created_at", "updated_at", "is_online"
        ];

        for field in expected_fields {
            assert!(
                user_data.get(field).is_some(),
                "Field '{}' should be present in response",
                field
            );
        }

        // Verify sensitive fields are not present
        assert!(user_data.get("password").is_none(), "Password should not be in response");

        // Cleanup
        cleanup_user_by_email(target_user.email).await;
        cleanup_user_by_email(auth_user.email).await;
    }
}
