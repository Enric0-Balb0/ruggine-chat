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
use crate::common::{cleanup_user_by_id, cleanup_group_chat, cleanup_group_membership};
use axum::body::to_bytes;

#[cfg(test)]
mod find_by_user_id_e2e_tests {
    use ruggine_server::utils::service_initializer::ServiceInitializer;
    use crate::{clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id, create_full_router, get_database};
    use super::*;

    /// Helper function to create test data with multiple group memberships and login user
    async fn setup_test_scenario_with_multiple_groups(prefix: &str) -> (i32, i32, Vec<i32>, Vec<i32>, String) {
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

        // Create multiple group chats
        let mut group_chat_ids = Vec::new();
        let mut membership_ids = Vec::new();

        for i in 0..3 {
            // Create group chat
            let group_chat_dto = GroupChatFactory::unique_fake_group_chat_create_dto(&format!("{}_group_{}", prefix, i));
            let group_chat = group_chat_service.create(group_chat_dto, admin_user.id).await.unwrap();
            group_chat_ids.push(group_chat.id);

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
            let membership = group_membership_service.find_by_user_id_and_group_id(member_user.id, group_chat.id).await.unwrap();
            membership_ids.push(membership.id);
        }

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

        (admin_user.id, member_user.id, group_chat_ids, membership_ids, token)
    }

    /// Helper function to create test data with single group membership
    async fn setup_test_scenario_single_group(prefix: &str) -> (i32, i32, i32, i32, String) {
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
    async fn test_find_by_user_id_e2e_success_multiple_memberships() {
        // Arrange: Set up test scenario with multiple groups
        let (admin_id, member_id, group_chat_ids, membership_ids, token) = setup_test_scenario_with_multiple_groups("find_by_user_id_multiple").await;
        let app = create_full_router().await;

        // Act: Send GET request to find all group memberships for user
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/user")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 with array of memberships
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let memberships = json["data"].as_array().unwrap();
        assert_eq!(memberships.len(), 3); // Should have 3 memberships

        // Check that all memberships belong to the correct user
        for membership in memberships {
            assert_eq!(membership["user_id"].as_i64().unwrap() as i32, member_id);
            assert_eq!(membership["membership_status"].as_str().unwrap(), "active");
            
            // Check that the membership ID and group chat ID are in our expected lists
            let membership_id = membership["id"].as_i64().unwrap() as i32;
            let group_chat_id = membership["group_chat_id"].as_i64().unwrap() as i32;
            assert!(membership_ids.contains(&membership_id));
            assert!(group_chat_ids.contains(&group_chat_id));
        }

        // Cleanup
        for group_chat_id in group_chat_ids {
            clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_id, group_chat_id).await;
            clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(member_id, group_chat_id).await;
            cleanup_group_chat(group_chat_id).await;
        }
        cleanup_user_by_id(admin_id).await;
        cleanup_user_by_id(member_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_e2e_success_single_membership() {
        // Arrange: Set up test scenario with single group
        let (admin_id, member_id, group_chat_id, membership_id, token) = setup_test_scenario_single_group("find_by_user_id_single").await;
        let app = create_full_router().await;

        // Act: Send GET request to find all group memberships for user
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/user")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 with array containing one membership
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let memberships = json["data"].as_array().unwrap();
        assert_eq!(memberships.len(), 1); // Should have 1 membership

        let membership = &memberships[0];
        assert_eq!(membership["id"].as_i64().unwrap() as i32, membership_id);
        assert_eq!(membership["user_id"].as_i64().unwrap() as i32, member_id);
        assert_eq!(membership["group_chat_id"].as_i64().unwrap() as i32, group_chat_id);
        assert_eq!(membership["membership_status"].as_str().unwrap(), "active");

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_id, group_chat_id).await;
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(member_id, group_chat_id).await;
        cleanup_group_chat(group_chat_id).await;
        cleanup_user_by_id(admin_id).await;
        cleanup_user_by_id(member_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_e2e_success_empty_result() {
        // Arrange: Create a user with no group memberships
        let db = get_database().await;
        let user_service = UserService::new(&db);
        let user_dto = UserFactory::unique_fake_user_register_dto("find_by_user_id_empty");
        let user_password = user_dto.password.clone();
        let user = user_service.create_user(user_dto.clone()).await.unwrap();

        // Login user to get token
        let app = create_full_router().await;
        let login_payload = json!({
            "email": user_dto.email,
            "password": user_password
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
        let token = login_json["data"]["token"].as_str().unwrap().to_string();

        // Act: Send GET request to find all group memberships for user
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/user")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 with empty array
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let memberships = json["data"].as_array().unwrap();
        assert_eq!(memberships.len(), 0); // Should have no memberships

        // Cleanup
        cleanup_user_by_id(user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_e2e_unauthorized_no_token() {
        // Arrange: Create router
        let app = create_full_router().await;

        // Act: Send GET request without authorization header
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/user")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_e2e_success_with_left_membership() {
        // Arrange: Set up test scenario and leave one group
        let (admin_id, member_id, group_chat_id, membership_id, token) = setup_test_scenario_single_group("find_by_user_id_left").await;
        
        // First, leave the group
        let app = create_full_router().await;
        let leave_payload = json!({
            "id": membership_id
        });

        let leave_request = Request::builder()
            .method("PATCH")
            .uri("/api/group_membership/leave")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token.clone()))
            .body(Body::from(leave_payload.to_string()))
            .unwrap();

        let leave_response = app.clone().oneshot(leave_request).await.unwrap();
        assert_eq!(leave_response.status(), StatusCode::OK);

        // Act: Send GET request to find all group memberships for user
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/user")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 with array containing the left membership
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let memberships = json["data"].as_array().unwrap();
        assert_eq!(memberships.len(), 0); // Should not retrieve the left membership record

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(admin_id, group_chat_id).await;
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(member_id, group_chat_id).await;
        cleanup_group_chat(group_chat_id).await;
        cleanup_user_by_id(admin_id).await;
        cleanup_user_by_id(member_id).await;
    }
}
