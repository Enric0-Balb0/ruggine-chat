use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use axum::body::to_bytes;
use crate::common::{cleanup_user_by_email, cleanup_group_chat, cleanup_invitation, create_invitation_router, create_login_and_get_token, create_test_group_chat, create_test_invitation};

#[cfg(test)]
mod find_by_user_id_invitation_e2e_tests {
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_invitation_success_received_invitations() {
        // Arrange: Create router, users, group chats, and invitations
        let app = create_invitation_router().await;
        let (sender1, _password1, _token1) = create_login_and_get_token("e2e_find_by_user_id_sender1".to_string()).await;
        let (sender2, _password2, _token2) = create_login_and_get_token("e2e_find_by_user_id_sender2".to_string()).await;
        let (recipient_user, _password3, recipient_token) = create_login_and_get_token("e2e_find_by_user_id_recipient".to_string()).await;
        
        let group_chat1 = create_test_group_chat("e2e_find_by_user_id_group1", sender1.id).await;
        let group_chat2 = create_test_group_chat("e2e_find_by_user_id_group2", sender2.id).await;
        
        // Create invitations where recipient_user is the recipient
        let invitation1 = create_test_invitation(sender1.id, recipient_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(sender2.id, recipient_user.id, group_chat2.id).await;

        // Act: Send GET request to /user with recipient's token
        let request = Request::builder()
            .method("GET")
            .uri("/user")
            .header("authorization", format!("Bearer {}", recipient_token))
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
        let data = response_json["data"].as_array().unwrap();
        
        assert_eq!(data.len(), 2, "Should return 2 received invitations");
        
        // Verify both invitations are for the recipient user
        for invitation_data in data {
            assert_eq!(invitation_data["to_user_id"].as_i64().unwrap(), recipient_user.id as i64);
            assert!(invitation_data["sent_at"].as_str().is_some(), "Should have sent_at timestamp");
            assert!(invitation_data["responded_at"].is_null(), "Should have null responded_at for pending invitation");
            assert_eq!(invitation_data["status"].as_str().unwrap(), "pending");
        }
        
        // Verify invitation IDs match
        let returned_ids: Vec<i64> = data.iter().map(|inv| inv["id"].as_i64().unwrap()).collect();
        assert!(returned_ids.contains(&(invitation1.id as i64)));
        assert!(returned_ids.contains(&(invitation2.id as i64)));

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user_by_email(sender1.email).await;
        cleanup_user_by_email(sender2.email).await;
        cleanup_user_by_email(recipient_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_invitation_no_invitations() {
        // Arrange: Create router and user with no invitations
        let app = create_invitation_router().await;
        let (user_with_no_invitations, _password, token) = create_login_and_get_token("e2e_find_by_user_id_empty".to_string()).await;

        // Act: Send GET request to /user
        let request = Request::builder()
            .method("GET")
            .uri("/user")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with empty array
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure and data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = response_json["data"].as_array().unwrap();
        
        assert_eq!(data.len(), 0, "Should return empty array for user with no invitations");

        // Cleanup
        cleanup_user_by_email(user_with_no_invitations.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_invitation_only_received_not_sent() {
        // Arrange: Create router, users where target user sends invitations (not receives)
        let app = create_invitation_router().await;
        let (sender_user, _password1, sender_token) = create_login_and_get_token("e2e_find_by_user_id_sender_only".to_string()).await;
        let (recipient1, _password2, _token2) = create_login_and_get_token("e2e_find_by_user_id_recipient1".to_string()).await;
        let (recipient2, _password3, _token3) = create_login_and_get_token("e2e_find_by_user_id_recipient2".to_string()).await;
        
        let group_chat = create_test_group_chat("e2e_find_by_user_id_sender_group", sender_user.id).await;
        
        // Create invitations where sender_user is the sender (not recipient)
        let invitation1 = create_test_invitation(sender_user.id, recipient1.id, group_chat.id).await;
        let invitation2 = create_test_invitation(sender_user.id, recipient2.id, group_chat.id).await;

        // Act: Send GET request to /user with sender's token (should only return received invitations)
        let request = Request::builder()
            .method("GET")
            .uri("/user")
            .header("authorization", format!("Bearer {}", sender_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with empty array (since sender_user has no received invitations)
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure and data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = response_json["data"].as_array().unwrap();
        
        assert_eq!(data.len(), 0, "Should return empty array since user only sent invitations, didn't receive any");

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(sender_user.email).await;
        cleanup_user_by_email(recipient1.email).await;
        cleanup_user_by_email(recipient2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_invitation_mixture_scenario() {
        // Arrange: Create router, users for mixture scenario
        let app = create_invitation_router().await;
        let (sender_user, _password1, _sender_token) = create_login_and_get_token("e2e_find_by_user_id_mixture_sender".to_string()).await;
        let (recipient1, _password2, recipient1_token) = create_login_and_get_token("e2e_find_by_user_id_mixture_recipient1".to_string()).await;
        let (recipient2, _password3, _recipient2_token) = create_login_and_get_token("e2e_find_by_user_id_mixture_recipient2".to_string()).await;

        let group_chat = create_test_group_chat("e2e_find_by_user_id_mixture_group", sender_user.id).await;

        // Create invitations where sender_user is the sender
        let invitation1 = create_test_invitation(sender_user.id, recipient1.id, group_chat.id).await;
        let invitation2 = create_test_invitation(sender_user.id, recipient2.id, group_chat.id).await;

        // Act: Send GET request to /user with recipient1's token (should return only one invitation)
        let request = Request::builder()
            .method("GET")
            .uri("/user")
            .header("authorization", format!("Bearer {}", recipient1_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with one invitation
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure and data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = response_json["data"].as_array().unwrap();
        
        assert_eq!(data.len(), 1, "Should return only one invitation");

        // Verify invitation is for recipient1
        assert_eq!(data[0]["to_user_id"].as_i64().unwrap(), recipient1.id as i64);
        
        // Verify invitation ID matches
        let returned_id = data[0]["id"].as_i64().unwrap();
        assert_eq!(returned_id, invitation1.id as i64);

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(sender_user.email).await;
        cleanup_user_by_email(recipient1.email).await;
        cleanup_user_by_email(recipient2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_invitation_different_statuses() {
        // Arrange: Create router, users, and test different invitation statuses
        let app = create_invitation_router().await;
        let (sender, _password1, _sender_token) = create_login_and_get_token("e2e_find_by_user_id_status_sender".to_string()).await;
        let (recipient_user, _password2, recipient_token) = create_login_and_get_token("e2e_find_by_user_id_status_recipient".to_string()).await;
        
        let group_chat = create_test_group_chat("e2e_find_by_user_id_status_group", sender.id).await;
        
        // Create invitation
        let invitation = create_test_invitation(sender.id, recipient_user.id, group_chat.id).await;

        // Update invitation status to accepted by making an API call
        let update_request_body = serde_json::json!({
            "status": "accepted",
            "invitation_id": invitation.id
        });

        let update_request = Request::builder()
            .method("PATCH")
            .uri("/update-status")
            .header("authorization", format!("Bearer {}", recipient_token))
            .header("content-type", "application/json")
            .body(Body::from(update_request_body.to_string()))
            .unwrap();

        let update_response = app.clone().oneshot(update_request).await.unwrap();
        assert_eq!(update_response.status(), StatusCode::OK);

        // Act: Send GET request to /user after status update
        let request = Request::builder()
            .method("GET")
            .uri("/user")
            .header("authorization", format!("Bearer {}", recipient_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with accepted invitation
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure and data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = response_json["data"].as_array().unwrap();
        
        assert_eq!(data.len(), 1, "Should return 1 invitation");
        assert_eq!(data[0]["status"].as_str().unwrap(), "accepted", "Should return accepted invitation");
        assert_eq!(data[0]["to_user_id"].as_i64().unwrap(), recipient_user.id as i64);

        // Cleanup (invitation status changed creates group membership, need special cleanup)
        use crate::common::cleanup_test_user_from_a_group_chat;
        cleanup_test_user_from_a_group_chat(recipient_user.id, group_chat.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(recipient_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_invitation_ordering() {
        // Arrange: Create router, users, and multiple invitations to test ordering
        let app = create_invitation_router().await;
        let (sender1, _password1, _sender1_token) = create_login_and_get_token("e2e_find_by_user_id_order_sender1".to_string()).await;
        let (sender2, _password2, _sender2_token) = create_login_and_get_token("e2e_find_by_user_id_order_sender2".to_string()).await;
        let (recipient_user, _password3, recipient_token) = create_login_and_get_token("e2e_find_by_user_id_order_recipient".to_string()).await;
        
        let group_chat = create_test_group_chat("e2e_find_by_user_id_order_group", sender1.id).await;
        
        // Create invitations with some delay to ensure different sent_at times
        let invitation1 = create_test_invitation(sender1.id, recipient_user.id, group_chat.id).await;
        
        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let invitation2 = create_test_invitation(sender2.id, recipient_user.id, group_chat.id).await;

        // Act: Send GET request to /user
        let request = Request::builder()
            .method("GET")
            .uri("/user")
            .header("authorization", format!("Bearer {}", recipient_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return invitations ordered by sent_at DESC
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure and data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = response_json["data"].as_array().unwrap();
        
        assert_eq!(data.len(), 2, "Should return 2 invitations");
        
        // Parse timestamps to verify ordering (most recent first)
        let sent_at_0 = data[0]["sent_at"].as_str().unwrap();
        let sent_at_1 = data[1]["sent_at"].as_str().unwrap();
        
        // Convert to comparable format
        use chrono::{DateTime, Utc};
        let timestamp_0: DateTime<Utc> = sent_at_0.parse().unwrap();
        let timestamp_1: DateTime<Utc> = sent_at_1.parse().unwrap();
        
        assert!(timestamp_0 >= timestamp_1, "Invitations should be ordered by sent_at DESC");

        // Cleanup
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(sender1.email).await;
        cleanup_user_by_email(sender2.email).await;
        cleanup_user_by_email(recipient_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_invitation_response_structure() {
        // Arrange: Create router and simple invitation to test response structure
        let app = create_invitation_router().await;
        let (sender, _password1, _sender_token) = create_login_and_get_token("e2e_find_by_user_id_struct_sender".to_string()).await;
        let (recipient_user, _password2, recipient_token) = create_login_and_get_token("e2e_find_by_user_id_struct_recipient".to_string()).await;
        
        let group_chat = create_test_group_chat("e2e_find_by_user_id_struct_group", sender.id).await;
        let invitation = create_test_invitation(sender.id, recipient_user.id, group_chat.id).await;

        // Act: Send GET request to /user
        let request = Request::builder()
            .method("GET")
            .uri("/user")
            .header("authorization", format!("Bearer {}", recipient_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Verify response structure and DTO mapping
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure and data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = response_json["data"].as_array().unwrap();
        
        assert_eq!(data.len(), 1, "Should return 1 invitation");
        
        let returned_invitation = &data[0];
        assert_eq!(returned_invitation["id"].as_i64().unwrap(), invitation.id as i64);
        assert_eq!(returned_invitation["from_user_id"].as_i64().unwrap(), invitation.from_user_id as i64);
        assert_eq!(returned_invitation["to_user_id"].as_i64().unwrap(), invitation.to_user_id as i64);
        assert_eq!(returned_invitation["group_chat_id"].as_i64().unwrap(), invitation.group_chat_id as i64);
        assert_eq!(returned_invitation["status"].as_str().unwrap(), "pending");
        assert!(returned_invitation["sent_at"].as_str().is_some(), "Should have sent_at timestamp");
        assert!(returned_invitation["responded_at"].is_null(), "Should have null responded_at for pending invitation");

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(recipient_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_invitation_unauthorized() {
        // Arrange: Create router
        let app = create_invitation_router().await;

        // Act: Send GET request to /user without authorization header
        let request = Request::builder()
            .method("GET")
            .uri("/user")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_invitation_invalid_token() {
        // Arrange: Create router
        let app = create_invitation_router().await;

        // Act: Send GET request to /user with invalid authorization header
        let request = Request::builder()
            .method("GET")
            .uri("/user")
            .header("authorization", "Bearer invalid_token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
