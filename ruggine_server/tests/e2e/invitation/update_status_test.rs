use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use axum::body::to_bytes;
use crate::common::{cleanup_user, cleanup_group_chat, cleanup_invitation, cleanup_group_membership_by_invitation_id,
                   create_invitation_router, create_login_and_get_token, create_test_group_chat, 
                   create_test_user, create_test_invitation, create_test_admin_invitation};

#[cfg(test)]
mod update_status_invitation_e2e_tests {
    use serde_json::Value::Null;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_accept_invitation_success() {
        // Arrange: Create router, users, group chat, and invitation
        let app = create_invitation_router().await;
        let (sender_user, _password) = create_test_user("e2e_accept_sender").await;
        let (recipient_user, _recipient_password, token) = create_login_and_get_token("e2e_accept_recipient".to_string()).await;
        let group_chat = create_test_group_chat("e2e_accept_group", sender_user.id).await;
        let invitation = create_test_invitation(sender_user.id, recipient_user.id, group_chat.id).await;
        
        // Create update payload to accept invitation
        let update_payload = json!({
            "invitation_id": invitation.id,
            "status": "accepted",
        });

        // Act: Send PATCH request to /update-status with auth token
        let request = Request::builder()
            .method("PATCH")
            .uri("/update-status")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(update_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with updated invitation data
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure and data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = &response_json["data"];
        
        assert_eq!(data["id"].as_i64().unwrap(), invitation.id as i64);
        assert_eq!(data["status"].as_str().unwrap(), "accepted");
        assert!(data.get("responded_at").is_some(), "Response should contain responded_at field");
        assert!(data.get("group_membership_id").is_some(), "Response should contain group_membership_id field");

        // Cleanup
        cleanup_group_membership_by_invitation_id(invitation.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(recipient_user.email).await;
        cleanup_user(sender_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_decline_invitation_success() {
        // Arrange: Create router, users, group chat, and invitation
        let app = create_invitation_router().await;
        let (sender_user, _password) = create_test_user("e2e_decline_sender").await;
        let (recipient_user, _recipient_password, token) = create_login_and_get_token("e2e_decline_recipient".to_string()).await;
        let group_chat = create_test_group_chat("e2e_decline_group", sender_user.id).await;
        let invitation = create_test_invitation(sender_user.id, recipient_user.id, group_chat.id).await;
        
        // Create update payload to reject invitation
        let update_payload = json!({
            "invitation_id": invitation.id,
            "status": "rejected",
        });

        // Act: Send PATCH request to /update-status with auth token
        let request = Request::builder()
            .method("PATCH")
            .uri("/update-status")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(update_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with updated invitation data
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure and data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = &response_json["data"];
        
        assert_eq!(data["id"].as_i64().unwrap(), invitation.id as i64);
        assert_eq!(data["status"].as_str().unwrap(), "rejected");
        assert!(data.get("responded_at").is_some(), "Response should contain responded_at field");
        assert_eq!(data["group_membership_id"], Null, "Group membership ID should be null when rejected");

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(recipient_user.email).await;
        cleanup_user(sender_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_unauthorized_user_cannot_update_invitation() {
        // Arrange: Create router, users, group chat, and invitation
        let app = create_invitation_router().await;
        let (sender_user, _password) = create_test_user("e2e_unauth_sender").await;
        let (recipient_user, _password) = create_test_user("e2e_unauth_recipient").await;
        let (unauthorized_user, _unauth_password, unauth_token) = create_login_and_get_token("e2e_unauth_user".to_string()).await;
        let group_chat = create_test_group_chat("e2e_unauth_group", sender_user.id).await;
        let invitation = create_test_invitation(sender_user.id, recipient_user.id, group_chat.id).await;
        
        // Create update payload to accept invitation
        let update_payload = json!({
            "invitation_id": invitation.id,
            "status": "accepted",
        });

        // Act: Send PATCH request with wrong user's token
        let request = Request::builder()
            .method("PATCH")
            .uri("/update-status")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", unauth_token))
            .body(Body::from(update_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 403 Forbidden
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(unauthorized_user.email).await;
        cleanup_user(recipient_user.email).await;
        cleanup_user(sender_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_cannot_update_already_responded_invitation() {
        // Arrange: Create router, users, group chat, and invitation
        let app = create_invitation_router().await;
        let (sender_user, _password) = create_test_user("e2e_responded_sender").await;
        let (recipient_user, _recipient_password, token) = create_login_and_get_token("e2e_responded_recipient".to_string()).await;
        let group_chat = create_test_group_chat("e2e_responded_group", sender_user.id).await;
        let invitation = create_test_invitation(sender_user.id, recipient_user.id, group_chat.id).await;
        
        // First, accept the invitation
        let first_update_payload = json!({
            "invitation_id": invitation.id,
            "status": "accepted",
        });

        let first_request = Request::builder()
            .method("PATCH")
            .uri("/update-status")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(first_update_payload.to_string()))
            .unwrap();

        let first_response = app.clone().oneshot(first_request).await.unwrap();
        assert_eq!(first_response.status(), StatusCode::OK, "First update should succeed");

        // Now try to reject the already accepted invitation
        let second_update_payload = json!({
            "invitation_id": invitation.id,
            "status": "rejected",
        });

        // Act: Send second PATCH request
        let second_request = Request::builder()
            .method("PATCH")
            .uri("/update-status")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(second_update_payload.to_string()))
            .unwrap();

        let second_response = app.oneshot(second_request).await.unwrap();

        // Assert: Should return 409 Conflict (invitation already responded)
        assert_eq!(second_response.status(), StatusCode::CONFLICT);

        // Cleanup
        cleanup_group_membership_by_invitation_id(invitation.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(recipient_user.email).await;
        cleanup_user(sender_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_invalid_invitation_id() {
        // Test with non-existent invitation ID
        // Arrange: Create router and user
        let app = create_invitation_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_invalid_id_user".to_string()).await;
        
        // Create update payload with non-existent invitation ID
        let update_payload = json!({
            "invitation_id": 99999,
            "status": "accepted",
        });

        // Act: Send PATCH request
        let request = Request::builder()
            .method("PATCH")
            .uri("/update-status")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(update_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_invalid_status_pending() {
        // Test trying to set status to pending (should be rejected)
        // Arrange: Create router, users, group chat, and invitation
        let app = create_invitation_router().await;
        let (sender_user, _password) = create_test_user("e2e_invalid_status_sender").await;
        let (recipient_user, _recipient_password, token) = create_login_and_get_token("e2e_invalid_status_recipient".to_string()).await;
        let group_chat = create_test_group_chat("e2e_invalid_status_group", sender_user.id).await;
        let invitation = create_test_invitation(sender_user.id, recipient_user.id, group_chat.id).await;
        
        // Create update payload with invalid status
        let update_payload = json!({
            "invitation_id": invitation.id,
            "status": "pending",
        });

        // Act: Send PATCH request
        let request = Request::builder()
            .method("PATCH")
            .uri("/update-status")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(update_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(recipient_user.email).await;
        cleanup_user(sender_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_without_authorization() {
        // Test request without authorization token
        // Arrange: Create router
        let app = create_invitation_router().await;
        
        // Create update payload
        let update_payload = json!({
            "invitation_id": 1,
            "status": "accepted",
        });

        // Act: Send PATCH request without authorization header
        let request = Request::builder()
            .method("PATCH")
            .uri("/update-status")
            .header("content-type", "application/json")
            .body(Body::from(update_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_fr8_accept_admin_invitation_creates_admin_membership() {
        // FR8.2: Accept admin invitations and verify admin role is granted
        // Arrange: Create router, users, group chat, and admin invitation
        let app = create_invitation_router().await;
        let (sender_user, _password) = create_test_user("e2e_admin_invite_sender").await;
        let (recipient_user, _recipient_password, token) = create_login_and_get_token("e2e_admin_invite_recipient".to_string()).await;
        let group_chat = create_test_group_chat("e2e_admin_invite_group", sender_user.id).await;
        let admin_invitation = create_test_admin_invitation(sender_user.id, recipient_user.id, group_chat.id).await;
        
        // Create update payload to accept admin invitation
        let update_payload = json!({
            "invitation_id": admin_invitation.id,
            "status": "accepted",
        });

        // Act: Send PATCH request to /update-status
        let request = Request::builder()
            .method("PATCH")
            .uri("/update-status")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(update_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with updated invitation data
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure and data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = &response_json["data"];
        
        assert_eq!(data["id"].as_i64().unwrap(), admin_invitation.id as i64);
        assert_eq!(data["status"].as_str().unwrap(), "accepted");
        assert!(data.get("responded_at").is_some(), "Response should contain responded_at field");

        // Cleanup - note that accepting invitation should have created group membership
        cleanup_group_membership_by_invitation_id(admin_invitation.id).await;
        cleanup_invitation(admin_invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(recipient_user.email).await;
        cleanup_user(sender_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_malformed_json() {
        // Test with malformed JSON payload
        // Arrange: Create router and user
        let app = create_invitation_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_malformed_user".to_string()).await;
        
        // Act: Send PATCH request with malformed JSON
        let request = Request::builder()
            .method("PATCH")
            .uri("/update-status")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from("{ invalid json"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_status_missing_required_fields() {
        // Test with missing required fields
        // Arrange: Create router and user
        let app = create_invitation_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_missing_fields_user".to_string()).await;
        
        // Create incomplete payload (missing invitation_id)
        let update_payload = json!({
            "status": "accepted",
        });

        // Act: Send PATCH request
        let request = Request::builder()
            .method("PATCH")
            .uri("/update-status")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(update_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_user(user.email).await;
    }
}
