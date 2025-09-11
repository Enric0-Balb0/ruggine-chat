use crate::common::{cleanup_group_chat, cleanup_user};
use axum::body::to_bytes;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use ruggine_server::entity::invitation::InvitationStatus;
use ruggine_server::factory::{
    group_chat_factory::GroupChatFactory,
    invitation_factory::InvitationFactory,
    user_factory::UserFactory
};
use ruggine_server::service::{
    group_chat_service::GroupChatServiceTrait,
    group_membership_service::GroupMembershipServiceTrait,
    invitation_service::InvitationServiceTrait,
    user_service::UserServiceTrait
};
use serde_json::json;
use tower::ServiceExt;

#[cfg(test)]
mod find_online_users_in_group_e2e_tests {
    use super::*;
    use crate::{cleanup_test_user_from_a_group_chat, create_full_router, get_database};
    use ruggine_server::utils::service_initializer::ServiceInitializer;

    /// Helper function to create test scenario with a group containing online and offline users
    async fn setup_test_scenario_with_group_users(prefix: &str) -> (i32, i32, i32, i32, i32, i32, String) {
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let user_service = service_init.user_service();
        let group_chat_service = service_init.group_chat_service();
        let invitation_service = service_init.invitation_service();

        // Create admin user (group owner)
        let admin_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_admin", prefix));
        let admin_user = user_service.create_user(admin_dto.clone()).await.unwrap();

        // Create auth user (the one who will make the API call)
        let auth_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_auth", prefix));
        let auth_password = auth_dto.password.clone();
        let auth_user = user_service.create_user(auth_dto.clone()).await.unwrap();

        // Create online user 1
        let online_user1_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_online1", prefix));
        let online_user1 = user_service.create_user(online_user1_dto.clone()).await.unwrap();

        // Create online user 2
        let online_user2_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_online2", prefix));
        let online_user2 = user_service.create_user(online_user2_dto.clone()).await.unwrap();

        // Create offline user
        let offline_user_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_offline", prefix));
        let offline_user = user_service.create_user(offline_user_dto.clone()).await.unwrap();

        // Create group chat
        let group_chat_dto = GroupChatFactory::unique_fake_group_chat_create_dto(&format!("{}_group", prefix));
        let group_chat = group_chat_service.create(group_chat_dto, admin_user.id).await.unwrap();

        // Create invitations and memberships for all users
        for user_id in [auth_user.id, online_user1.id, online_user2.id, offline_user.id] {
            let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(user_id, group_chat.id);
            let invitation = invitation_service.send(invitation_dto, admin_user.id).await.unwrap();
            
            let update_dto = ruggine_server::dto::invitation_dto::InvitationUpdateStatusDto {
                invitation_id: invitation.id,
                status: InvitationStatus::Accepted,
            };
            let _ = invitation_service.update_status(update_dto, user_id).await.unwrap();
        }

        // Set online status for users
        let update_online_dto_true = UserFactory::fake_update_online_dto_true();
        let update_offline_dto_false = UserFactory::fake_update_online_dto_false();

        user_service.update_online(auth_user.id, update_online_dto_true.clone()).await.unwrap();
        user_service.update_online(online_user1.id, update_online_dto_true.clone()).await.unwrap();
        user_service.update_online(online_user2.id, update_online_dto_true).await.unwrap();
        user_service.update_online(offline_user.id, update_offline_dto_false).await.unwrap();

        // Login auth user to get token
        let app = create_full_router().await;
        let login_payload = json!({
            "email": auth_dto.email,
            "password": auth_password
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

        (admin_user.id, auth_user.id, online_user1.id, online_user2.id, offline_user.id, group_chat.id, token)
    }

    /// Helper function to create test scenario with auth user not in group
    async fn setup_test_scenario_auth_not_member(prefix: &str) -> (i32, i32, i32, i32, String) {
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let user_service = service_init.user_service();
        let group_chat_service = service_init.group_chat_service();
        let invitation_service = service_init.invitation_service();

        // Create admin user (group owner)
        let admin_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_admin", prefix));
        let admin_user = user_service.create_user(admin_dto.clone()).await.unwrap();

        // Create auth user (NOT a member of the group)
        let auth_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_auth", prefix));
        let auth_password = auth_dto.password.clone();
        let auth_user = user_service.create_user(auth_dto.clone()).await.unwrap();

        // Create member user
        let member_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_member", prefix));
        let member_user = user_service.create_user(member_dto.clone()).await.unwrap();

        // Create group chat
        let group_chat_dto = GroupChatFactory::unique_fake_group_chat_create_dto(&format!("{}_group", prefix));
        let group_chat = group_chat_service.create(group_chat_dto, admin_user.id).await.unwrap();

        // Add only member_user to the group (NOT auth_user)
        let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(member_user.id, group_chat.id);
        let invitation = invitation_service.send(invitation_dto, admin_user.id).await.unwrap();
        
        let update_dto = ruggine_server::dto::invitation_dto::InvitationUpdateStatusDto {
            invitation_id: invitation.id,
            status: InvitationStatus::Accepted,
        };
        let _ = invitation_service.update_status(update_dto, member_user.id).await.unwrap();

        // Set both users online
        let update_online_dto_true = UserFactory::fake_update_online_dto_true();
        user_service.update_online(auth_user.id, update_online_dto_true.clone()).await.unwrap();
        user_service.update_online(member_user.id, update_online_dto_true).await.unwrap();

        // Login auth user to get token
        let app = create_full_router().await;
        let login_payload = json!({
            "email": auth_dto.email,
            "password": auth_password
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

        (admin_user.id, auth_user.id, member_user.id, group_chat.id, token)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_e2e_success_multiple_online() {
        // Arrange: Set up test scenario with multiple online and offline users
        let (admin_id, auth_id, online_user1_id, online_user2_id, offline_user_id, group_chat_id, token) 
            = setup_test_scenario_with_group_users("find_online_group_success").await;
        let app = create_full_router().await;

        // Act: Send GET request to find online users in group
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/online-users-in-group/{}", group_chat_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 with online users
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let online_user_ids = json["data"].as_array().unwrap();
        assert_eq!(online_user_ids.len(), 3, "Should find 3 online users (auth + 2 online users)");

        let user_ids: Vec<i32> = online_user_ids
            .iter()
            .map(|id| id.as_i64().unwrap() as i32)
            .collect();

        assert!(user_ids.contains(&auth_id), "Should contain auth user");
        assert!(user_ids.contains(&online_user1_id), "Should contain online user 1");
        assert!(user_ids.contains(&online_user2_id), "Should contain online user 2");
        assert!(!user_ids.contains(&offline_user_id), "Should not contain offline user");

        // Cleanup
        cleanup_test_user_from_a_group_chat(auth_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(online_user1_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(online_user2_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(offline_user_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id).await;
        cleanup_group_chat(group_chat_id).await;
        cleanup_user(admin_id).await;
        cleanup_user(auth_id).await;
        cleanup_user(online_user1_id).await;
        cleanup_user(online_user2_id).await;
        cleanup_user(offline_user_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_e2e_only_auth_user_online() {
        // Arrange: Set up scenario where only auth user is online
        let (admin_id, auth_id, online_user1_id, online_user2_id, offline_user_id, group_chat_id, token) 
            = setup_test_scenario_with_group_users("find_online_group_only_auth").await;

        // Set all other users offline
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let user_service = service_init.user_service();
        let update_offline_dto = UserFactory::fake_update_online_dto_false();
        
        user_service.update_online(online_user1_id, update_offline_dto.clone()).await.unwrap();
        user_service.update_online(online_user2_id, update_offline_dto.clone()).await.unwrap();
        user_service.update_online(offline_user_id, update_offline_dto).await.unwrap();

        let app = create_full_router().await;

        // Act: Send GET request to find online users in group
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/online-users-in-group/{}", group_chat_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 with only auth user
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let online_user_ids = json["data"].as_array().unwrap();
        assert_eq!(online_user_ids.len(), 1, "Should find only auth user online");

        let user_ids: Vec<i32> = online_user_ids
            .iter()
            .map(|id| id.as_i64().unwrap() as i32)
            .collect();

        assert!(user_ids.contains(&auth_id), "Should contain only auth user");

        // Cleanup
        cleanup_test_user_from_a_group_chat(auth_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(online_user1_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(online_user2_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(offline_user_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id).await;
        cleanup_group_chat(group_chat_id).await;
        cleanup_user(admin_id).await;
        cleanup_user(auth_id).await;
        cleanup_user(online_user1_id).await;
        cleanup_user(online_user2_id).await;
        cleanup_user(offline_user_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_e2e_auth_user_offline_error() {
        // Arrange: Set up scenario and set auth user offline
        let (admin_id, auth_id, online_user1_id, online_user2_id, offline_user_id, group_chat_id, token) 
            = setup_test_scenario_with_group_users("find_online_group_auth_offline").await;

        // Set auth user offline
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let user_service = service_init.user_service();
        let update_offline_dto = UserFactory::fake_update_online_dto_false();
        user_service.update_online(auth_id, update_offline_dto).await.unwrap();

        let app = create_full_router().await;

        // Act: Send GET request to find online users in group
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/online-users-in-group/{}", group_chat_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 403 Forbidden because auth user is not online
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Cleanup
        cleanup_test_user_from_a_group_chat(auth_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(online_user1_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(online_user2_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(offline_user_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id).await;
        cleanup_group_chat(group_chat_id).await;
        cleanup_user(admin_id).await;
        cleanup_user(auth_id).await;
        cleanup_user(online_user1_id).await;
        cleanup_user(online_user2_id).await;
        cleanup_user(offline_user_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_e2e_auth_user_not_member_error() {
        // Arrange: Set up scenario where auth user is not a member
        let (admin_id, auth_id, member_id, group_chat_id, token) 
            = setup_test_scenario_auth_not_member("find_online_group_not_member").await;
        let app = create_full_router().await;

        // Act: Send GET request to find online users in group
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/online-users-in-group/{}", group_chat_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found because auth user is not a member
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Cleanup
        cleanup_test_user_from_a_group_chat(member_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id).await;
        cleanup_group_chat(group_chat_id).await;
        cleanup_user(admin_id).await;
        cleanup_user(auth_id).await;
        cleanup_user(member_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_e2e_nonexistent_group() {
        // Arrange: Set up auth user but use non-existent group
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let user_service = service_init.user_service();

        let auth_dto = UserFactory::unique_fake_user_register_dto("find_online_group_nonexistent_auth");
        let auth_password = auth_dto.password.clone();
        let auth_user = user_service.create_user(auth_dto.clone()).await.unwrap();

        let update_online_dto = UserFactory::fake_update_online_dto_true();
        user_service.update_online(auth_user.id, update_online_dto).await.unwrap();

        // Login to get token
        let app = create_full_router().await;
        let login_payload = json!({
            "email": auth_dto.email,
            "password": auth_password
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

        let nonexistent_group_id = 999999;

        // Act: Send GET request to find online users in non-existent group
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/online-users-in-group/{}", nonexistent_group_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Cleanup
        cleanup_user(auth_user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_e2e_unauthorized() {
        // Arrange: Create app without authentication
        let app = create_full_router().await;

        // Act: Send GET request without token
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/online-users-in-group/1")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_e2e_invalid_token() {
        // Arrange: Create app with invalid token
        let app = create_full_router().await;

        // Act: Send GET request with invalid token
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/online-users-in-group/1")
            .header("authorization", "Bearer invalid_token_here")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_e2e_dynamic_status_changes() {
        // Arrange: Set up initial scenario
        let (admin_id, auth_id, online_user1_id, online_user2_id, offline_user_id, group_chat_id, token) 
            = setup_test_scenario_with_group_users("find_online_group_dynamic").await;
        let app = create_full_router().await;

        // Test 1: Initial state with mixed online/offline users
        let request1 = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/online-users-in-group/{}", group_chat_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response1 = app.clone().oneshot(request1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::OK);

        let body1 = to_bytes(response1.into_body(), usize::MAX).await.unwrap();
        let json1: serde_json::Value = serde_json::from_slice(&body1).unwrap();
        let online_user_ids1 = json1["data"].as_array().unwrap();
        assert_eq!(online_user_ids1.len(), 3, "Should initially have 3 online users");

        // Test 2: Set offline_user to online
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let user_service = service_init.user_service();
        
        let update_online_dto = UserFactory::fake_update_online_dto_true();
        user_service.update_online(offline_user_id, update_online_dto).await.unwrap();

        let request2 = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/online-users-in-group/{}", group_chat_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response2 = app.clone().oneshot(request2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::OK);

        let body2 = to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let json2: serde_json::Value = serde_json::from_slice(&body2).unwrap();
        let online_user_ids2 = json2["data"].as_array().unwrap();
        assert_eq!(online_user_ids2.len(), 4, "Should now have 4 online users");

        // Test 3: Set one online user to offline
        let update_offline_dto = UserFactory::fake_update_online_dto_false();
        user_service.update_online(online_user1_id, update_offline_dto).await.unwrap();

        let request3 = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/online-users-in-group/{}", group_chat_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response3 = app.clone().oneshot(request3).await.unwrap();
        assert_eq!(response3.status(), StatusCode::OK);

        let body3 = to_bytes(response3.into_body(), usize::MAX).await.unwrap();
        let json3: serde_json::Value = serde_json::from_slice(&body3).unwrap();
        let online_user_ids3 = json3["data"].as_array().unwrap();
        assert_eq!(online_user_ids3.len(), 3, "Should be back to 3 online users");

        // Verify the correct users are included in final result
        let final_user_ids: Vec<i32> = online_user_ids3
            .iter()
            .map(|id| id.as_i64().unwrap() as i32)
            .collect();

        assert!(final_user_ids.contains(&auth_id), "Should contain auth user");
        assert!(final_user_ids.contains(&online_user2_id), "Should contain online user 2");
        assert!(final_user_ids.contains(&offline_user_id), "Should contain previously offline user (now online)");
        assert!(!final_user_ids.contains(&online_user1_id), "Should not contain user that was set offline");

        // Cleanup
        cleanup_test_user_from_a_group_chat(auth_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(online_user1_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(online_user2_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(offline_user_id, group_chat_id).await;
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id).await;
        cleanup_group_chat(group_chat_id).await;
        cleanup_user(admin_id).await;
        cleanup_user(auth_id).await;
        cleanup_user(online_user1_id).await;
        cleanup_user(online_user2_id).await;
        cleanup_user(offline_user_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_online_users_in_group_e2e_invalid_group_id_parameter() {
        // Arrange: Set up auth user
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let user_service = service_init.user_service();

        let auth_dto = UserFactory::unique_fake_user_register_dto("find_online_group_invalid_param_auth");
        let auth_password = auth_dto.password.clone();
        let auth_user = user_service.create_user(auth_dto.clone()).await.unwrap();

        let update_online_dto = UserFactory::fake_update_online_dto_true();
        user_service.update_online(auth_user.id, update_online_dto).await.unwrap();

        // Login to get token
        let app = create_full_router().await;
        let login_payload = json!({
            "email": auth_dto.email,
            "password": auth_password
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

        // Act: Send GET request with invalid group_id parameter
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/online-users-in-group/invalid_id")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request or similar for invalid parameter
        assert!(
            response.status() == StatusCode::BAD_REQUEST || 
            response.status() == StatusCode::NOT_FOUND,
            "Expected 400 or 404 for invalid parameter, got: {:?}", 
            response.status()
        );

        // Cleanup
        cleanup_user(auth_user.id).await;
    }
}
