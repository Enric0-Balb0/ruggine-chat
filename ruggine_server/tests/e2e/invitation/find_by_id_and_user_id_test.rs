use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use axum::body::to_bytes;
use crate::common::{cleanup_user_by_email, cleanup_group_chat, cleanup_invitation, create_invitation_router, create_login_and_get_token, create_test_group_chat, create_test_user, create_test_invitation};

#[cfg(test)]
mod find_by_id_and_user_id_invitation_e2e_tests {
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_invitation_success_as_from_user() {
        // Arrange: Create router, users, group chat, and invitation
        let app = create_invitation_router().await;
        let (admin_user, _password, token) = create_login_and_get_token("e2e_find_by_id_and_user_id_success_admin".to_string()).await;
        let group_chat = create_test_group_chat("e2e_find_by_id_and_user_id_success_group", admin_user.id).await;
        let (target_user, _target_password) = create_test_user("e2e_find_by_id_and_user_id_success_target").await;
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;

        // Act: Send GET request to /{id} with from_user auth token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/{}", invitation.id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
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
        
        assert_eq!(data["id"].as_i64().unwrap(), invitation.id as i64);
        assert_eq!(data["from_user_id"].as_i64().unwrap(), admin_user.id as i64);
        assert_eq!(data["to_user_id"].as_i64().unwrap(), target_user.id as i64);
        assert_eq!(data["group_chat_id"].as_i64().unwrap(), group_chat.id as i64);
        assert_eq!(data["status"].as_str().unwrap(), "pending");
        assert!(data["sent_at"].as_str().is_some(), "Should have sent_at timestamp");
        assert!(data["responded_at"].is_null(), "Should have null responded_at for pending invitation");

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_invitation_success_as_to_user() {
        // Arrange: Create router, users, group chat, and invitation
        let app = create_invitation_router().await;
        let (admin_user, _password, _token) = create_login_and_get_token("e2e_find_by_id_and_user_id_to_user_admin".to_string()).await;
        let group_chat = create_test_group_chat("e2e_find_by_id_and_user_id_to_user_group", admin_user.id).await;
        let (target_user, _target_password, target_token) = create_login_and_get_token("e2e_find_by_id_and_user_id_to_user_target".to_string()).await;
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;

        // Act: Send GET request to /{id} with to_user auth token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/{}", invitation.id))
            .header("authorization", format!("Bearer {}", target_token))
            .body(Body::empty())
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
        
        assert_eq!(data["id"].as_i64().unwrap(), invitation.id as i64);
        assert_eq!(data["from_user_id"].as_i64().unwrap(), admin_user.id as i64);
        assert_eq!(data["to_user_id"].as_i64().unwrap(), target_user.id as i64);
        assert_eq!(data["group_chat_id"].as_i64().unwrap(), group_chat.id as i64);

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_invitation_unauthorized_no_token() {
        // Arrange: Create router, users, group chat, and invitation
        let app = create_invitation_router().await;
        let (admin_user, _password, _token) = create_login_and_get_token("e2e_find_by_id_and_user_id_no_auth_admin".to_string()).await;
        let group_chat = create_test_group_chat("e2e_find_by_id_and_user_id_no_auth_group", admin_user.id).await;
        let (target_user, _target_password) = create_test_user("e2e_find_by_id_and_user_id_no_auth_target").await;
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;

        // Act: Send GET request without auth token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/{}", invitation.id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_invitation_unauthorized_invalid_token() {
        // Arrange: Create router, users, group chat, and invitation
        let app = create_invitation_router().await;
        let (admin_user, _password, _token) = create_login_and_get_token("e2e_find_by_id_and_user_id_invalid_auth_admin".to_string()).await;
        let group_chat = create_test_group_chat("e2e_find_by_id_and_user_id_invalid_auth_group", admin_user.id).await;
        let (target_user, _target_password) = create_test_user("e2e_find_by_id_and_user_id_invalid_auth_target").await;
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;

        // Act: Send GET request with invalid token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/{}", invitation.id))
            .header("authorization", "Bearer invalid_token_here")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(target_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_invitation_not_found_invalid_id() {
        // Arrange: Create router and user
        let app = create_invitation_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_find_by_id_and_user_id_not_found_user".to_string()).await;
        let non_existent_id = 99999;

        // Act: Send GET request with non-existent invitation ID
        let request = Request::builder()
            .method("GET")
            .uri(format!("/{}", non_existent_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_invitation_not_found_unauthorized_user() {
        // Arrange: Create router, users, group chat, and invitation
        let app = create_invitation_router().await;
        let (admin_user, _password, _token) = create_login_and_get_token("e2e_find_by_id_and_user_id_unauthorized_admin".to_string()).await;
        let group_chat = create_test_group_chat("e2e_find_by_id_and_user_id_unauthorized_group", admin_user.id).await;
        let (target_user, _target_password) = create_test_user("e2e_find_by_id_and_user_id_unauthorized_target").await;
        let invitation = create_test_invitation(admin_user.id, target_user.id, group_chat.id).await;
        
        // Create a third user who is not involved in the invitation
        let (random_user, _random_password, random_token) = create_login_and_get_token("e2e_find_by_id_and_user_id_unauthorized_random".to_string()).await;

        // Act: Send GET request with uninvolved user token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/{}", invitation.id))
            .header("authorization", format!("Bearer {}", random_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found (because user is not authorized to see this invitation)
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(admin_user.email).await;
        cleanup_user_by_email(target_user.email).await;
        cleanup_user_by_email(random_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_invitation_bad_request_invalid_id_format() {
        // Arrange: Create router and user
        let app = create_invitation_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_find_by_id_and_user_id_invalid_format_user".to_string()).await;

        // Act: Send GET request with invalid ID format
        let request = Request::builder()
            .method("GET")
            .uri("/not_a_number")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_invitation_bad_request_negative_id() {
        // Arrange: Create router and user
        let app = create_invitation_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_find_by_id_and_user_id_negative_id_user".to_string()).await;

        // Act: Send GET request with negative ID
        let request = Request::builder()
            .method("GET")
            .uri("/-1")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found (or could be 400 Bad Request depending on implementation)
        assert!(
            response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::BAD_REQUEST,
            "Expected 404 or 400, got {}", response.status()
        );

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_and_user_id_invitation_returns_different_invitations() {
        // Arrange: Create router, users, group chats, and invitations
        let app = create_invitation_router().await;
        let (admin_user1, _password1, token1) = create_login_and_get_token("e2e_find_by_id_and_user_id_different_admin1".to_string()).await;
        let group_chat1 = create_test_group_chat("e2e_find_by_id_and_user_id_different_group1", admin_user1.id).await;
        let (target_user1, _target_password1) = create_test_user("e2e_find_by_id_and_user_id_different_target1").await;
        let invitation1 = create_test_invitation(admin_user1.id, target_user1.id, group_chat1.id).await;
        
        let (admin_user2, _password2, token2) = create_login_and_get_token("e2e_find_by_id_and_user_id_different_admin2".to_string()).await;
        let group_chat2 = create_test_group_chat("e2e_find_by_id_and_user_id_different_group2", admin_user2.id).await;
        let (target_user2, _target_password2) = create_test_user("e2e_find_by_id_and_user_id_different_target2").await;
        let invitation2 = create_test_invitation(admin_user2.id, target_user2.id, group_chat2.id).await;

        // Act: Find first invitation
        let request1 = Request::builder()
            .method("GET")
            .uri(format!("/{}", invitation1.id))
            .header("authorization", format!("Bearer {}", token1))
            .body(Body::empty())
            .unwrap();

        let response1 = app.clone().oneshot(request1).await.unwrap();
        
        // Act: Find second invitation
        let request2 = Request::builder()
            .method("GET")
            .uri(format!("/{}", invitation2.id))
            .header("authorization", format!("Bearer {}", token2))
            .body(Body::empty())
            .unwrap();

        let response2 = app.oneshot(request2).await.unwrap();

        // Assert: Both should succeed with correct data
        assert_eq!(response1.status(), StatusCode::OK);
        assert_eq!(response2.status(), StatusCode::OK);

        let body1 = to_bytes(response1.into_body(), usize::MAX).await.unwrap();
        let response1_text = String::from_utf8(body1.to_vec()).unwrap();
        let response1_json: serde_json::Value = serde_json::from_str(&response1_text).unwrap();
        let data1 = &response1_json["data"];
        
        let body2 = to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let response2_text = String::from_utf8(body2.to_vec()).unwrap();
        let response2_json: serde_json::Value = serde_json::from_str(&response2_text).unwrap();
        let data2 = &response2_json["data"];
        
        assert_eq!(data1["id"].as_i64().unwrap(), invitation1.id as i64);
        assert_eq!(data1["from_user_id"].as_i64().unwrap(), admin_user1.id as i64);
        assert_eq!(data1["to_user_id"].as_i64().unwrap(), target_user1.id as i64);
        
        assert_eq!(data2["id"].as_i64().unwrap(), invitation2.id as i64);
        assert_eq!(data2["from_user_id"].as_i64().unwrap(), admin_user2.id as i64);
        assert_eq!(data2["to_user_id"].as_i64().unwrap(), target_user2.id as i64);
        
        assert_ne!(data1["id"].as_i64().unwrap(), data2["id"].as_i64().unwrap());

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user_by_email(admin_user1.email).await;
        cleanup_user_by_email(target_user1.email).await;
        cleanup_user_by_email(admin_user2.email).await;
        cleanup_user_by_email(target_user2.email).await;
    }
}
