use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use axum::body::to_bytes;
use serde_json::json;
use crate::common::{
    cleanup_user_by_email, cleanup_group_chat, cleanup_text_message, 
    create_text_message_router, create_login_and_get_token, 
    create_test_group_chat_with_invitation_and_membership,
    cleanup_test_user_from_a_group_chat, create_test_group_chat
};

#[cfg(test)]
mod create_text_message_e2e_tests {
    use crate::leave_user_from_a_group;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_success() {
        // Arrange: Create router, user, and group
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_create_msg_success".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_create_msg_group", user.id).await;
        
        let payload = json!({
            "content": "Test message from e2e test",
            "group_chat_id": group.id
        });

        // Act: Send POST request to /create with auth token
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with created message
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        
        let message_data = &response_json["data"];
        assert!(message_data["id"].as_i64().is_some(), "Message should have an ID");
        assert_eq!(message_data["content"].as_str().unwrap(), "Test message from e2e test");
        assert_eq!(message_data["sender_id"].as_i64().unwrap(), user.id as i64);
        assert_eq!(message_data["group_chat_id"].as_i64().unwrap(), group.id as i64);
        assert!(message_data["sent_at"].as_str().is_some(), "Message should have sent_at timestamp");

        let message_id = message_data["id"].as_i64().unwrap() as i32;

        // Cleanup
        cleanup_text_message(message_id).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_unauthorized_no_token() {
        // Arrange: Create router and prepare payload
        let app = create_text_message_router().await;
        
        let payload = json!({
            "content": "Test message without auth",
            "group_chat_id": 1
        });

        // Act: Send POST request without auth token
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_invalid_token() {
        // Arrange: Create router and prepare payload
        let app = create_text_message_router().await;
        
        let payload = json!({
            "content": "Test message with invalid token",
            "group_chat_id": 1
        });

        // Act: Send POST request with invalid auth token
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("authorization", "Bearer invalid_token_12345")
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_empty_content() {
        // Arrange: Create router, user, and group
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_create_empty_content".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_create_empty_group", user.id).await;
        
        let payload = json!({
            "content": "",
            "group_chat_id": group.id
        });

        // Act: Send POST request with empty content
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request (validation error)
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_nonexistent_group() {
        // Arrange: Create router and user
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_create_no_group".to_string()).await;
        
        let nonexistent_group_id = 99999;
        let payload = json!({
            "content": "Test message for nonexistent group",
            "group_chat_id": nonexistent_group_id
        });

        // Act: Send POST request for nonexistent group
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 403 Forbidden (user cannot access messages)
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_user_not_member() {
        // Arrange: Create router, two users, and group where sender is not a member
        let app = create_text_message_router().await;
        let (sender, _password, token) = create_login_and_get_token("e2e_create_not_member".to_string()).await;
        let (group_owner, _password_owner, _token_owner) = create_login_and_get_token("e2e_create_group_owner".to_string()).await;
        
        // Create group with group_owner as member, but sender is not a member
        let group = create_test_group_chat("e2e_create_private_group", group_owner.id).await;
        
        let payload = json!({
            "content": "Test message from non-member",
            "group_chat_id": group.id
        });

        // Act: Send POST request from user who is not a member
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 403 Forbidden (user cannot access messages)
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Cleanup
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(group_owner.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_user_not_active_member() {
        // Arrange: Create router and user
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_create_not_active".to_string()).await;
        
        // Create a test group with invitation and membership for the user
        let group = create_test_group_chat_with_invitation_and_membership("e2e_create_left_group", user.id).await;
        
        // Make the user leave the group (set membership status to Left)
        leave_user_from_a_group(user.id, group.id).await;
        
        let payload = json!({
            "content": "Test message from non-active member",
            "group_chat_id": group.id
        });

        // Act: Send POST request from user who left the group
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 403 Forbidden (user cannot access messages)
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Cleanup
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_invalid_json() {
        // Arrange: Create router and user
        let app = create_text_message_router().await;
        let (_user, _password, token) = create_login_and_get_token("e2e_create_invalid_json".to_string()).await;

        // Act: Send POST request with invalid JSON
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from("{ invalid json }"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_user_by_email(_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_missing_fields() {
        // Arrange: Create router and user
        let app = create_text_message_router().await;
        let (_user, _password, token) = create_login_and_get_token("e2e_create_missing_fields".to_string()).await;

        // Act: Send POST request with missing group_chat_id field
        let payload = json!({
            "content": "Test message with missing group_chat_id"
        });

        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_user_by_email(_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_with_long_content() {
        // Arrange: Create router, user, and group
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_create_long_content".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_create_long_group", user.id).await;
        
        // Create message with long content (but within limits)
        let long_content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
        let payload = json!({
            "content": long_content,
            "group_chat_id": group.id
        });

        // Act: Send POST request with long content
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let message_data = &response_json["data"];
        let message_id = message_data["id"].as_i64().unwrap() as i32;
        assert_eq!(message_data["content"].as_str().unwrap(), long_content);

        // Cleanup
        cleanup_text_message(message_id).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_with_special_characters() {
        // Arrange: Create router, user, and group
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_create_special_chars".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_create_special_group", user.id).await;
        
        let special_content = "Message with émojis 🎉 and special chars: àáâãäå çčć";
        let payload = json!({
            "content": special_content,
            "group_chat_id": group.id
        });

        // Act: Send POST request with special characters
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let message_data = &response_json["data"];
        let message_id = message_data["id"].as_i64().unwrap() as i32;
        assert_eq!(message_data["content"].as_str().unwrap(), special_content);

        // Cleanup
        cleanup_text_message(message_id).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }
}
