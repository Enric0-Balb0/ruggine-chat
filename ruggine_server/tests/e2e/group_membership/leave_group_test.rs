use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use ruggine_server::service::{
    user_service::{UserService, UserServiceTrait},
    group_chat_service::{GroupChatServiceTrait},
    invitation_service::{InvitationServiceTrait},
    group_membership_service::{GroupMembershipServiceTrait}
};
use ruggine_server::factory::{
    user_factory::UserFactory,
    group_chat_factory::GroupChatFactory,
    invitation_factory::InvitationFactory
};
use ruggine_server::entity::{
    invitation::InvitationStatus
};
use crate::common::{cleanup_user, cleanup_group_chat, cleanup_group_membership};
use axum::body::to_bytes;

#[cfg(test)]
mod leave_group_e2e_tests {
    use ruggine_server::utils::service_initializer::ServiceInitializer;
    use crate::{cleanup_test_user_from_a_group_chat, create_full_router, get_database};
    use super::*;

    /// Helper function to create test data and login user
    async fn setup_test_scenario(prefix: &str) -> (i32, i32, i32, i32, String) {
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let user_service = service_init.user_service();
        let group_chat_service = service_init.group_chat_service();
        let invitation_service = service_init.invitation_service();
        let group_membership_service = service_init.group_membership_service();

        // Create admin user
        let admin_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_admin", prefix));
        let admin_password = admin_dto.password.clone();
        let admin_user = user_service.create_user(admin_dto.clone()).await.unwrap();

        // Create member user  
        let member_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_member", prefix));
        let member_password = member_dto.password.clone();
        let member_user = user_service.create_user(member_dto.clone()).await.unwrap();

        // Create group chat
        let group_chat_dto = GroupChatFactory::unique_fake_group_chat_create_dto(&format!("{}_group", prefix));
        let group_chat = group_chat_service.create(group_chat_dto, admin_user.id).await.unwrap();

        // Create invitation
        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(member_user.id, group_chat.id);
        let invitation = invitation_service.send(invitation_dto, admin_user.id).await.unwrap();

        // Accept invitation to create membership
        let update_dto = ruggine_server::dto::invitation_dto::InvitationUpdateStatusDto {
            invitation_id: invitation.id,
            status: InvitationStatus::Accepted,
        };
        let _ = invitation_service.update_status(update_dto, member_user.id).await.unwrap();

        // Find the created membership
        let memberships = group_membership_service.find_by_user_id(member_user.id).await.unwrap();
        let membership_id = memberships[0].id;

        // Login member user to get token
        let app = create_full_router().await;
        let login_payload = json!({
            "email": member_dto.email,
            "password": member_password
        });

        let login_request = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(login_payload.to_string()))
            .unwrap();

        let login_response = app.clone().oneshot(login_request).await.unwrap();
        assert_eq!(login_response.status(), StatusCode::OK);

        let login_body = to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
        let login_json: serde_json::Value = serde_json::from_slice(&login_body).unwrap();
        let token = login_json["data"]["token"].as_str().unwrap().to_string();

        (admin_user.id, member_user.id, group_chat.id, membership_id, token)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_e2e_success() {
        // Arrange: Set up test scenario
        let (admin_id, member_id, group_chat_id, membership_id, token) = setup_test_scenario("leave_success").await;
        let app = create_full_router().await;

        let leave_payload = json!({
            "id": membership_id
        });

        // Act: Send PUT request to leave group
        let request = Request::builder()
            .method("PATCH")
            .uri("/api/group_membership/leave")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(leave_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 with updated membership
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // assert!(json["success"].as_bool().unwrap());
        assert_eq!(json["data"]["id"].as_i64().unwrap() as i32, membership_id);
        assert_eq!(json["data"]["user_id"].as_i64().unwrap() as i32, member_id);
        assert_eq!(json["data"]["membership_status"].as_str().unwrap(), "left");
        assert!(json["data"]["left_at"].as_str().is_some());

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(member_id, group_chat_id).await;
        cleanup_group_chat(group_chat_id).await;
        cleanup_user(admin_id).await;
        cleanup_user(member_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_e2e_unauthorized_no_token() {
        // Arrange: Create router
        let app = create_full_router().await;

        let leave_payload = json!({
            "id": 1
        });

        // Act: Send PUT request without authorization header
        let request = Request::builder()
            .method("PATCH")
            .uri("/api/group_membership/leave")
            .header("content-type", "application/json")
            .body(Body::from(leave_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_e2e_membership_not_found() {
        // Arrange: Set up test scenario
        let (admin_id, member_id, group_chat_id, membership_id, token) = setup_test_scenario("leave_not_found").await;
        let app = create_full_router().await;

        let leave_payload = json!({
            "id": 999999  // Non-existent membership ID
        });

        // Act: Send PUT request to leave group
        let request = Request::builder()
            .method("PATCH")
            .uri("/api/group_membership/leave")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(leave_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(member_id, group_chat_id).await;
        cleanup_group_membership(membership_id).await;
        cleanup_group_chat(group_chat_id).await;
        cleanup_user(admin_id).await;
        cleanup_user(member_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_e2e_already_left() {
        // Arrange: Set up test scenario
        let (admin_id, member_id, group_chat_id, membership_id, token) = setup_test_scenario("leave_already_left").await;
        let app = create_full_router().await;

        let leave_payload = json!({
            "id": membership_id
        });

        // Act: First leave request (should succeed)
        let first_request = Request::builder()
            .method("PATCH")
            .uri("/api/group_membership/leave")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token.clone()))
            .body(Body::from(leave_payload.to_string()))
            .unwrap();

        let first_response = app.clone().oneshot(first_request).await.unwrap();
        assert_eq!(first_response.status(), StatusCode::OK);

        // Act: Second leave request (should fail)
        let second_request = Request::builder()
            .method("PATCH")
            .uri("/api/group_membership/leave")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(leave_payload.to_string()))
            .unwrap();

        let second_response = app.oneshot(second_request).await.unwrap();

        // Assert: Should return 409 Conflict
        assert_eq!(second_response.status(), StatusCode::CONFLICT);

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(member_id, group_chat_id).await;
        cleanup_group_membership(membership_id).await;
        cleanup_group_chat(group_chat_id).await;
        cleanup_user(admin_id).await;
        cleanup_user(member_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_e2e_invalid_payload() {
        // Arrange: Set up test scenario (just need a token)
        let (admin_id, member_id, group_chat_id, membership_id, token) = setup_test_scenario("leave_invalid").await;
        let app = create_full_router().await;

        let invalid_payload = json!({
            "id": -1  // Invalid ID (should fail validation)
        });

        // Act: Send PUT request with invalid payload
        let request = Request::builder()
            .method("PATCH")
            .uri("/api/group_membership/leave")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(invalid_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(member_id, group_chat_id).await;
        cleanup_group_membership(membership_id).await;
        cleanup_group_chat(group_chat_id).await;
        cleanup_user(admin_id).await;
        cleanup_user(member_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_leave_group_e2e_unauthorized_different_user() {
        // Arrange: Set up test scenario and create another user
        let (admin_id, member_id, group_chat_id, membership_id, _token) = setup_test_scenario("leave_unauthorized").await;
        
        // Create another user and get their token
        let db = get_database().await;
        let user_service = UserService::new(&db);
        let other_user_dto = UserFactory::unique_fake_user_register_dto("leave_other_user");
        let other_password = other_user_dto.password.clone();
        let other_user = user_service.create_user(other_user_dto.clone()).await.unwrap();

        // Login other user to get token
        let app = create_full_router().await;
        let login_payload = json!({
            "email": other_user_dto.email,
            "password": other_password
        });

        let login_request = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header("content-type", "application/json")
            .body(Body::from(login_payload.to_string()))
            .unwrap();

        let login_response = app.clone().oneshot(login_request).await.unwrap();
        let login_body = to_bytes(login_response.into_body(), usize::MAX).await.unwrap();
        let login_json: serde_json::Value = serde_json::from_slice(&login_body).unwrap();
        let other_token = login_json["data"]["token"].as_str().unwrap().to_string();

        let leave_payload = json!({
            "id": membership_id
        });

        // Act: Send PUT request with other user's token
        let request = Request::builder()
            .method("PATCH")
            .uri("/api/group_membership/leave")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", other_token))
            .body(Body::from(leave_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found (for security, we don't reveal existence)
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(member_id, group_chat_id).await;
        cleanup_group_membership(membership_id).await;
        cleanup_group_chat(group_chat_id).await;
        cleanup_user(admin_id).await;
        cleanup_user(member_id).await;
        cleanup_user(other_user.id).await;
    }
}
