use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use axum::body::to_bytes;
use crate::common::{cleanup_user, cleanup_group_chat, cleanup_invitation, create_invitation_router, create_login_and_get_token, create_test_group_chat, create_test_user};

#[cfg(test)]
mod send_invitation_e2e_tests {
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_success_as_admin() {
        // Arrange: Create router, admin user, group chat, and target user
        let app = create_invitation_router().await;
        let (admin_user, _password, token) = create_login_and_get_token("e2e_send_success_admin".to_string()).await;
        let group_chat = create_test_group_chat("e2e_send_success_group", admin_user.id).await;
        let (target_user, _target_password) = create_test_user("e2e_send_success_target").await;
        
        // Create invitation payload
        let send_payload = json!({
            "to_user_id": target_user.id,
            "group_chat_id": group_chat.id
        });

        // Act: Send POST request to /send with auth token
        let request = Request::builder()
            .method("POST")
            .uri("/send")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(send_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with invitation data
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure and data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = &response_json["data"];
        
        assert_eq!(data["from_user_id"].as_i64().unwrap(), admin_user.id as i64);
        assert_eq!(data["to_user_id"].as_i64().unwrap(), target_user.id as i64);
        assert_eq!(data["group_chat_id"].as_i64().unwrap(), group_chat.id as i64);
        assert_eq!(data["status"].as_str().unwrap(), "pending");
        assert!(data["sent_at"].as_str().is_some(), "Should have sent_at timestamp");

        // Cleanup
        cleanup_invitation(data["id"].as_i64().unwrap() as i32).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_unauthorized_no_token() {
        // Arrange: Create router and invitation payload
        let app = create_invitation_router().await;
        let (admin_user, _password, _token) = create_login_and_get_token("e2e_send_no_auth_admin".to_string()).await;
        let group_chat = create_test_group_chat("e2e_send_no_auth_group", admin_user.id).await;
        let (target_user, _target_password) = create_test_user("e2e_send_no_auth_target").await;

        let send_payload = json!({
            "to_user_id": target_user.id,
            "group_chat_id": group_chat.id
        });

        // Act: Send POST request without auth token
        let request = Request::builder()
            .method("POST")
            .uri("/send")
            .header("content-type", "application/json")
            .body(Body::from(send_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_unauthorized_invalid_token() {
        // Arrange: Create router and invitation payload
        let app = create_invitation_router().await;
        let (admin_user, _password, _token) = create_login_and_get_token("e2e_send_invalid_auth_admin".to_string()).await;
        let group_chat = create_test_group_chat("e2e_send_invalid_auth_group", admin_user.id).await;
        let (target_user, _target_password) = create_test_user("e2e_send_invalid_auth_target").await;

        let send_payload = json!({
            "to_user_id": target_user.id,
            "group_chat_id": group_chat.id
        });

        // Act: Send POST request with invalid token
        let request = Request::builder()
            .method("POST")
            .uri("/send")
            .header("content-type", "application/json")
            .header("authorization", "Bearer invalid_token_here")
            .body(Body::from(send_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_forbidden_not_admin() {
        // Arrange: Create router, admin user, group chat, non-admin user, and target user
        let app = create_invitation_router().await;
        let (admin_user, _password, _token) = create_login_and_get_token("e2e_send_not_admin_admin".to_string()).await;
        let group_chat = create_test_group_chat("e2e_send_not_admin_group", admin_user.id).await;
        let (non_admin_user, _non_admin_password, non_admin_token) = create_login_and_get_token("e2e_send_not_admin_user".to_string()).await;
        let (target_user, _target_password) = create_test_user("e2e_send_not_admin_target").await;

        let send_payload = json!({
            "to_user_id": target_user.id,
            "group_chat_id": group_chat.id
        });

        // Act: Send POST request with non-admin user token
        let request = Request::builder()
            .method("POST")
            .uri("/send")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", non_admin_token))
            .body(Body::from(send_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 403 Forbidden
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Cleanup
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(non_admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_conflict_already_exists() {
        // Arrange: Create router, admin user, group chat, and target user
        let app = create_invitation_router().await;
        let (admin_user, _password, token) = create_login_and_get_token("e2e_send_duplicate_admin".to_string()).await;
        let group_chat = create_test_group_chat("e2e_send_duplicate_group", admin_user.id).await;
        let (target_user, _target_password) = create_test_user("e2e_send_duplicate_target").await;

        let send_payload = json!({
            "to_user_id": target_user.id,
            "group_chat_id": group_chat.id
        });

        // Send first invitation
        let request1 = Request::builder()
            .method("POST")
            .uri("/send")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(send_payload.to_string()))
            .unwrap();

        let response1 = app.clone().oneshot(request1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::OK, "First invitation should succeed");

        // Act: Send second invitation (duplicate)
        let request2 = Request::builder()
            .method("POST")
            .uri("/send")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(send_payload.to_string()))
            .unwrap();

        let response2 = app.oneshot(request2).await.unwrap();

        // Assert: Should return 409 Conflict
        assert_eq!(response2.status(), StatusCode::CONFLICT);

        let body = to_bytes(response1.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure and data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = &response_json["data"];

        // Cleanup
        cleanup_invitation(data["id"].as_i64().unwrap() as i32).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_not_found_target_user() {
        // Arrange: Create router, admin user, and group chat
        let app = create_invitation_router().await;
        let (admin_user, _password, token) = create_login_and_get_token("e2e_send_user_not_found_admin".to_string()).await;
        let group_chat = create_test_group_chat("e2e_send_user_not_found_group", admin_user.id).await;

        let send_payload = json!({
            "to_user_id": 99999, // Non-existent user ID
            "group_chat_id": group_chat.id
        });

        // Act: Send POST request with non-existent target user
        let request = Request::builder()
            .method("POST")
            .uri("/send")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(send_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        // Cleanup
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_not_found_group_chat() {
        // Arrange: Create router, admin user, and target user
        let app = create_invitation_router().await;
        let (admin_user, _password, token) = create_login_and_get_token("e2e_send_group_not_found_admin".to_string()).await;
        let (target_user, _target_password) = create_test_user("e2e_send_group_not_found_target").await;

        let send_payload = json!({
            "to_user_id": target_user.id,
            "group_chat_id": 99999 // Non-existent group chat ID
        });

        // Act: Send POST request with non-existent group chat
        let request = Request::builder()
            .method("POST")
            .uri("/send")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(send_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Cleanup
        cleanup_user(admin_user.email).await;
        cleanup_user(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_bad_request_invalid_data() {
        // Arrange: Create router and admin user
        let app = create_invitation_router().await;
        let (admin_user, _password, token) = create_login_and_get_token("e2e_send_invalid_data_admin".to_string()).await;

        // Invalid payload with negative IDs
        let send_payload = json!({
            "to_user_id": -1,
            "group_chat_id": -1
        });

        // Act: Send POST request with invalid data
        let request = Request::builder()
            .method("POST")
            .uri("/send")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(send_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_user(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_bad_request_missing_fields() {
        // Arrange: Create router and admin user
        let app = create_invitation_router().await;
        let (admin_user, _password, token) = create_login_and_get_token("e2e_send_missing_fields_admin".to_string()).await;

        // Payload missing required fields
        let send_payload = json!({
            "to_user_id": 1
            // Missing group_chat_id
        });

        // Act: Send POST request with missing fields
        let request = Request::builder()
            .method("POST")
            .uri("/send")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(send_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_user(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_send_invitation_bad_request_malformed_json() {
        // Arrange: Create router and admin user
        let app = create_invitation_router().await;
        let (admin_user, _password, token) = create_login_and_get_token("e2e_send_malformed_json_admin".to_string()).await;

        // Malformed JSON
        let malformed_payload = "{ \"to_user_id\": 1, \"group_chat_id\": }";

        // Act: Send POST request with malformed JSON
        let request = Request::builder()
            .method("POST")
            .uri("/send")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(malformed_payload))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_user(admin_user.email).await;
    }
}
