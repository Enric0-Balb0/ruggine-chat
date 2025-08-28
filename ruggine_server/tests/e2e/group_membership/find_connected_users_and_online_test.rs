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
use crate::common::{cleanup_user, cleanup_group_chat};
use axum::body::to_bytes;

#[cfg(test)]
mod find_connected_users_and_online_e2e_tests {
    use ruggine_server::utils::service_initializer::ServiceInitializer;
    use crate::{cleanup_test_user_from_a_group_chat, create_full_router, get_database};
    use super::*;

    /// Helper function to create test scenario with connected users (some online, some offline)
    async fn setup_test_scenario_with_connected_users(prefix: &str) -> (i32, i32, i32, i32, i32, i32, i32, String) {
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let user_service = service_init.user_service();
        let group_chat_service = service_init.group_chat_service();
        let invitation_service = service_init.invitation_service();

        // Create admin user (group owner)
        let admin_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_admin", prefix));
        let admin_user = user_service.create_user(admin_dto.clone()).await.unwrap();

        // Create target user (the one who will make the API call)
        let target_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_target", prefix));
        let target_password = target_dto.password.clone();
        let target_user = user_service.create_user(target_dto.clone()).await.unwrap();

        // Create online connected user 1
        let online_user1_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_online1", prefix));
        let online_user1 = user_service.create_user(online_user1_dto.clone()).await.unwrap();

        // Create online connected user 2
        let online_user2_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_online2", prefix));
        let online_user2 = user_service.create_user(online_user2_dto.clone()).await.unwrap();

        // Create offline connected user
        let offline_user_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_offline", prefix));
        let offline_user = user_service.create_user(offline_user_dto.clone()).await.unwrap();

        // Create group chats
        let group_chat1_dto = GroupChatFactory::unique_fake_group_chat_create_dto(&format!("{}_group1", prefix));
        let group_chat1 = group_chat_service.create(group_chat1_dto, admin_user.id).await.unwrap();

        let group_chat2_dto = GroupChatFactory::unique_fake_group_chat_create_dto(&format!("{}_group2", prefix));
        let group_chat2 = group_chat_service.create(group_chat2_dto, admin_user.id).await.unwrap();

        // Create invitations and memberships
        // Group 1: target_user, online_user1, offline_user
        for user_id in [target_user.id, online_user1.id, offline_user.id] {
            let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(user_id, group_chat1.id);
            let invitation = invitation_service.send(invitation_dto, admin_user.id).await.unwrap();
            
            let update_dto = ruggine_server::dto::invitation_dto::InvitationUpdateStatusDto {
                invitation_id: invitation.id,
                status: InvitationStatus::Accepted,
            };
            let _ = invitation_service.update_status(update_dto, user_id).await.unwrap();
        }

        // Group 2: target_user, online_user2
        for user_id in [target_user.id, online_user2.id] {
            let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(user_id, group_chat2.id);
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

        user_service.update_online(online_user1.id, update_online_dto_true.clone()).await.unwrap();
        user_service.update_online(online_user2.id, update_online_dto_true).await.unwrap();
        user_service.update_online(offline_user.id, update_offline_dto_false).await.unwrap();

        // Login target user to get token
        let app = create_full_router().await;
        let login_payload = json!({
            "email": target_dto.email,
            "password": target_password
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

        (admin_user.id, target_user.id, online_user1.id, online_user2.id, offline_user.id, group_chat1.id, group_chat2.id, token)
    }

    /// Helper function to create test scenario with no shared groups
    async fn setup_test_scenario_no_shared_groups(prefix: &str) -> (i32, i32, i32, i32, String) {
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let user_service = service_init.user_service();
        let group_chat_service = service_init.group_chat_service();
        let invitation_service = service_init.invitation_service();

        // Create admin users for separate groups
        let admin1_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_admin1", prefix));
        let admin1_user = user_service.create_user(admin1_dto.clone()).await.unwrap();

        let admin2_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_admin2", prefix));
        let admin2_user = user_service.create_user(admin2_dto.clone()).await.unwrap();

        // Create target user (who will make the API call)
        let target_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_target", prefix));
        let target_password = target_dto.password.clone();
        let target_user = user_service.create_user(target_dto.clone()).await.unwrap();

        // Create other user in different group
        let other_dto = UserFactory::unique_fake_user_register_dto(&format!("{}_other", prefix));
        let other_user = user_service.create_user(other_dto.clone()).await.unwrap();

        // Create separate group chats
        let group_chat1_dto = GroupChatFactory::unique_fake_group_chat_create_dto(&format!("{}_group1", prefix));
        let group_chat1 = group_chat_service.create(group_chat1_dto, admin1_user.id).await.unwrap();

        let group_chat2_dto = GroupChatFactory::unique_fake_group_chat_create_dto(&format!("{}_group2", prefix));
        let group_chat2 = group_chat_service.create(group_chat2_dto, admin2_user.id).await.unwrap();

        // Add target_user to group1, other_user to group2 (no shared groups)
        let invitation1_dto = InvitationFactory::fake_invitation_create_dto_with_ids(target_user.id, group_chat1.id);
        let invitation1 = invitation_service.send(invitation1_dto, admin1_user.id).await.unwrap();
        
        let update_dto1 = ruggine_server::dto::invitation_dto::InvitationUpdateStatusDto {
            invitation_id: invitation1.id,
            status: InvitationStatus::Accepted,
        };
        let _ = invitation_service.update_status(update_dto1, target_user.id).await.unwrap();

        let invitation2_dto = InvitationFactory::fake_invitation_create_dto_with_ids(other_user.id, group_chat2.id);
        let invitation2 = invitation_service.send(invitation2_dto, admin2_user.id).await.unwrap();
        
        let update_dto2 = ruggine_server::dto::invitation_dto::InvitationUpdateStatusDto {
            invitation_id: invitation2.id,
            status: InvitationStatus::Accepted,
        };
        let _ = invitation_service.update_status(update_dto2, other_user.id).await.unwrap();

        // Set other_user to online
        let update_online_dto = UserFactory::fake_update_online_dto_true();
        user_service.update_online(other_user.id, update_online_dto).await.unwrap();

        // Login target user to get token
        let app = create_full_router().await;
        let login_payload = json!({
            "email": target_dto.email,
            "password": target_password
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

        (target_user.id, other_user.id, group_chat1.id, group_chat2.id, token)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_and_online_e2e_success_mixed_status() {
        // Arrange: Set up test scenario with connected users (some online, some offline)
        let (admin_id, target_id, online_user1_id, online_user2_id, offline_user_id, group_chat_id1, group_chat_id2, token)
            = setup_test_scenario_with_connected_users("find_connected_online_mixed").await;
        let app = create_full_router().await;

        // Act: Send GET request to find connected users that are online
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/connected_users_online")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 with array of online user IDs
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let connected_user_ids = json["data"].as_array().unwrap();
        assert_eq!(connected_user_ids.len(), 2); // Should have 2 online users

        // Convert to Vec<i32> for easier checking
        let user_ids: Vec<i32> = connected_user_ids
            .iter()
            .map(|id| id.as_i64().unwrap() as i32)
            .collect();

        // Should contain both online users but not the offline user or self
        assert!(user_ids.contains(&online_user1_id));
        assert!(user_ids.contains(&online_user2_id));
        assert!(!user_ids.contains(&offline_user_id), "Should not include offline user");
        assert!(!user_ids.contains(&target_id), "Should not include self");

        // Cleanup
        cleanup_test_user_from_a_group_chat(target_id, group_chat_id1).await;
        cleanup_test_user_from_a_group_chat(target_id, group_chat_id2).await;
        cleanup_test_user_from_a_group_chat(online_user1_id, group_chat_id1).await;
        cleanup_test_user_from_a_group_chat(offline_user_id, group_chat_id1).await;
        cleanup_test_user_from_a_group_chat(online_user2_id, group_chat_id2).await;
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id1).await;
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id2).await;
        cleanup_group_chat(group_chat_id1).await;
        cleanup_group_chat(group_chat_id2).await;
        cleanup_user(admin_id).await;
        cleanup_user(target_id).await;
        cleanup_user(online_user1_id).await;
        cleanup_user(online_user2_id).await;
        cleanup_user(offline_user_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_and_online_e2e_no_shared_groups() {
        // Arrange: Set up test scenario with no shared groups
        let (target_id, other_id, group_chat1_id, group_chat2_id, token) 
            = setup_test_scenario_no_shared_groups("find_connected_online_no_shared").await;
        let app = create_full_router().await;

        // Act: Send GET request to find connected users that are online
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/connected_users_online")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 with empty array since no shared groups
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let connected_user_ids = json["data"].as_array().unwrap();
        assert_eq!(connected_user_ids.len(), 0, "Should have no connected users without shared groups");

        // Cleanup
        cleanup_test_user_from_a_group_chat(target_id, group_chat1_id).await;
        cleanup_test_user_from_a_group_chat(other_id, group_chat2_id).await;
        cleanup_group_chat(group_chat1_id).await;
        cleanup_group_chat(group_chat2_id).await;
        cleanup_user(target_id).await;
        cleanup_user(other_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_and_online_e2e_unauthorized() {
        // Arrange: Create app without authentication
        let app = create_full_router().await;

        // Act: Send GET request without token
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/connected_users_online")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_and_online_e2e_invalid_token() {
        // Arrange: Create app with invalid token
        let app = create_full_router().await;

        // Act: Send GET request with invalid token
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/connected_users_online")
            .header("authorization", "Bearer invalid_token_here")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_and_online_e2e_user_changes_status() {
        // Arrange: Set up initial scenario
        let (admin_id, target_id, online_user1_id, online_user2_id, offline_user_id, group_chat_id1, group_chat_id2, token)
            = setup_test_scenario_with_connected_users("find_connected_online_status_change").await;
        let app = create_full_router().await;

        // Test 1: Initial state with mixed online/offline users
        let request1 = Request::builder()
            .method("GET")
            .uri("/api/group_membership/connected_users_online")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response1 = app.clone().oneshot(request1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::OK);

        let body1 = to_bytes(response1.into_body(), usize::MAX).await.unwrap();
        let json1: serde_json::Value = serde_json::from_slice(&body1).unwrap();
        let connected_user_ids1 = json1["data"].as_array().unwrap();
        assert_eq!(connected_user_ids1.len(), 2, "Should initially have 2 online users");

        // Test 2: Set offline_user to online
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let user_service = service_init.user_service();
        
        let update_online_dto = UserFactory::fake_update_online_dto_true();
        user_service.update_online(offline_user_id, update_online_dto).await.unwrap();

        let request2 = Request::builder()
            .method("GET")
            .uri("/api/group_membership/connected_users_online")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response2 = app.clone().oneshot(request2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::OK);

        let body2 = to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let json2: serde_json::Value = serde_json::from_slice(&body2).unwrap();
        let connected_user_ids2 = json2["data"].as_array().unwrap();
        assert_eq!(connected_user_ids2.len(), 3, "Should now have 3 online users");

        // Test 3: Set one online user to offline
        let update_offline_dto = UserFactory::fake_update_online_dto_false();
        user_service.update_online(online_user1_id, update_offline_dto).await.unwrap();

        let request3 = Request::builder()
            .method("GET")
            .uri("/api/group_membership/connected_users_online")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response3 = app.clone().oneshot(request3).await.unwrap();
        assert_eq!(response3.status(), StatusCode::OK);

        let body3 = to_bytes(response3.into_body(), usize::MAX).await.unwrap();
        let json3: serde_json::Value = serde_json::from_slice(&body3).unwrap();
        let connected_user_ids3 = json3["data"].as_array().unwrap();
        assert_eq!(connected_user_ids3.len(), 2, "Should be back to 2 online users");

        // Verify the correct users are included in final result
        let final_user_ids: Vec<i32> = connected_user_ids3
            .iter()
            .map(|id| id.as_i64().unwrap() as i32)
            .collect();
        assert!(final_user_ids.contains(&online_user2_id), "Should include user2 who stayed online");
        assert!(final_user_ids.contains(&offline_user_id), "Should include previously offline user who went online");
        assert!(!final_user_ids.contains(&online_user1_id), "Should not include user1 who went offline");

        // Cleanup
        cleanup_test_user_from_a_group_chat(target_id, group_chat_id1).await;
        cleanup_test_user_from_a_group_chat(target_id, group_chat_id2).await;
        cleanup_test_user_from_a_group_chat(online_user1_id, group_chat_id1).await;
        cleanup_test_user_from_a_group_chat(online_user2_id, group_chat_id2).await;
        cleanup_test_user_from_a_group_chat(offline_user_id, group_chat_id1).await;
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id1).await;
        cleanup_test_user_from_a_group_chat(admin_id, group_chat_id2).await;
        cleanup_group_chat(group_chat_id1).await;
        cleanup_group_chat(group_chat_id2).await;
        cleanup_user(admin_id).await;
        cleanup_user(target_id).await;
        cleanup_user(online_user1_id).await;
        cleanup_user(online_user2_id).await;
        cleanup_user(offline_user_id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_connected_users_and_online_e2e_empty_result_all_offline() {
        // Arrange: Create a scenario where all connected users are offline
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let user_service = service_init.user_service();
        let group_chat_service = service_init.group_chat_service();
        let invitation_service = service_init.invitation_service();

        // Create admin user
        let admin_dto = UserFactory::unique_fake_user_register_dto("find_connected_all_offline_admin");
        let admin_user = user_service.create_user(admin_dto.clone()).await.unwrap();

        // Create target user
        let target_dto = UserFactory::unique_fake_user_register_dto("find_connected_all_offline_target");
        let target_password = target_dto.password.clone();
        let target_user = user_service.create_user(target_dto.clone()).await.unwrap();

        // Create connected user who will be offline
        let connected_dto = UserFactory::unique_fake_user_register_dto("find_connected_all_offline_connected");
        let connected_user = user_service.create_user(connected_dto.clone()).await.unwrap();

        // Create group chat and memberships
        let group_chat_dto = GroupChatFactory::unique_fake_group_chat_create_dto("find_connected_all_offline_group");
        let group_chat = group_chat_service.create(group_chat_dto, admin_user.id).await.unwrap();

        for user_id in [target_user.id, connected_user.id] {
            let invitation_dto = InvitationFactory::fake_invitation_create_dto_with_ids(user_id, group_chat.id);
            let invitation = invitation_service.send(invitation_dto, admin_user.id).await.unwrap();
            
            let update_dto = ruggine_server::dto::invitation_dto::InvitationUpdateStatusDto {
                invitation_id: invitation.id,
                status: InvitationStatus::Accepted,
            };
            let _ = invitation_service.update_status(update_dto, user_id).await.unwrap();
        }

        // Ensure connected user is offline
        let update_offline_dto = UserFactory::fake_update_online_dto_false();
        user_service.update_online(connected_user.id, update_offline_dto).await.unwrap();

        // Login target user
        let app = create_full_router().await;
        let login_payload = json!({
            "email": target_dto.email,
            "password": target_password
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

        // Act: Send GET request to find connected users that are online
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/connected_users_online")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 with empty array
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let connected_user_ids = json["data"].as_array().unwrap();
        assert_eq!(connected_user_ids.len(), 0, "Should have no online connected users");

        // Cleanup
        cleanup_test_user_from_a_group_chat(target_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(connected_user.id, group_chat.id).await;
        cleanup_test_user_from_a_group_chat(admin_user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(admin_user.id).await;
        cleanup_user(target_user.id).await;
        cleanup_user(connected_user.id).await;
    }
}
