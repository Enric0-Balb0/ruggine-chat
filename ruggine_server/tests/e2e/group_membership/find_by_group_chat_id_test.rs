use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use crate::common::{cleanup_user, cleanup_group_chat};
use axum::body::to_bytes;

#[cfg(test)]
mod find_by_group_chat_id_e2e_tests {
    use crate::{
        add_test_user_to_a_group, cleanup_test_user_from_a_group_chat, cleanup_test_users, create_full_router, create_login_and_get_token, create_test_group_chat_with_invitation_and_membership, create_test_user, create_test_users_for_a_group, get_database, test_user_leave_from_a_group
    };
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_e2e_success_multiple_members() {
        // Arrange: Create admin with group and add multiple members
        let (admin, _admin_pw, admin_token) = create_login_and_get_token("find_by_group_chat_id_multiple_admin".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("find_by_group_chat_id_multiple_group", admin.id).await;
        let members = create_test_users_for_a_group("find_by_group_chat_id_multiple_member", 3, &group).await;
        
        let app = create_full_router().await;

        // Act: Send GET request to find all memberships for the group
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/group_chat/{}", group.id))
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 with array of memberships
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let memberships = json["data"].as_array().unwrap();
        assert_eq!(memberships.len(), 4); // Admin + 3 members

        // Check that all memberships belong to the correct group
        let mut found_user_ids = Vec::new();
        for membership in memberships {
            assert_eq!(membership["group_chat_id"].as_i64().unwrap() as i32, group.id);
            assert_eq!(membership["membership_status"].as_str().unwrap(), "active");
            found_user_ids.push(membership["user_id"].as_i64().unwrap() as i32);
        }

        // Verify admin is included
        assert!(found_user_ids.contains(&admin.id));
        
        // Verify all members are included
        for (member, _, _) in &members {
            assert!(found_user_ids.contains(&member.id));
        }

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin.id, group.id).await;
        for (member, _, _) in members {
            cleanup_test_user_from_a_group_chat(member.id, group.id).await;
        }
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![admin.id]).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_e2e_success_single_member() {
        // Arrange: Create admin with group and add single member
        let (admin, _admin_pw, admin_token) = create_login_and_get_token("find_by_group_chat_id_single_admin".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("find_by_group_chat_id_single_group", admin.id).await;
        let members = create_test_users_for_a_group("find_by_group_chat_id_single_member", 1, &group).await;
        let (member, _, _) = &members[0];
        
        let app = create_full_router().await;

        // Act: Send GET request to find all memberships for the group
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/group_chat/{}", group.id))
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 with array containing two memberships (admin + member)
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let memberships = json["data"].as_array().unwrap();
        assert_eq!(memberships.len(), 2); // Admin + 1 member

        // Check that all memberships belong to the correct group
        let mut found_user_ids = Vec::new();
        for membership in memberships {
            assert_eq!(membership["group_chat_id"].as_i64().unwrap() as i32, group.id);
            assert_eq!(membership["membership_status"].as_str().unwrap(), "active");
            found_user_ids.push(membership["user_id"].as_i64().unwrap() as i32);
        }

        assert!(found_user_ids.contains(&admin.id));
        assert!(found_user_ids.contains(&member.id));

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin.id, group.id).await;
        cleanup_test_user_from_a_group_chat(member.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![admin.id, member.id]).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_e2e_unauthorized() {
        // Arrange: Create app without authentication
        let app = create_full_router().await;

        // Act: Send GET request without token
        let request = Request::builder()
            .method("GET")
            .uri("/api/group_membership/group_chat/123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_e2e_forbidden_not_member() {
        // Arrange: Create non-member user and admin with group
        let (non_member, _non_member_pw, non_member_token) = create_login_and_get_token("find_by_group_chat_id_non_member".to_string()).await;
        let (admin, _admin_pw, _admin_token) = create_login_and_get_token("find_by_group_chat_id_admin_forbidden".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("find_by_group_chat_id_forbidden_group", admin.id).await;

        let app = create_full_router().await;

        // Act: Send GET request to find memberships for group where user is not a member
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/group_chat/{}", group.id))
            .header("authorization", format!("Bearer {}", non_member_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found (user not member of group)
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![admin.id, non_member.id]).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_e2e_group_not_found() {
        // Arrange: Create user and login
        let (user, _user_pw, token) = create_login_and_get_token("find_by_group_chat_id_no_group".to_string()).await;
        let app = create_full_router().await;

        // Act: Send GET request for non-existent group
        let non_existent_group_id = 999999;
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/group_chat/{}", non_existent_group_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found (group doesn't exist)
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Cleanup
        cleanup_test_users(vec![user.id]).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_e2e_after_leaving_group() {
        // Arrange: Create admin with group and add member
        let (admin, _admin_pw, _admin_token) = create_login_and_get_token("find_by_group_chat_id_leave_admin".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("find_by_group_chat_id_leave_group", admin.id).await;
        let (member, member_pw, member_token) = create_login_and_get_token("find_by_group_chat_id_leave_member".to_string()).await;
        add_test_user_to_a_group(member.id, &group).await;

        let app = create_full_router().await;

        // First verify member can access group memberships
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/group_chat/{}", group.id))
            .header("authorization", format!("Bearer {}", member_token))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Member leaves the group
        test_user_leave_from_a_group(member.id, group.id).await;

        // Act: Try to access group memberships after leaving
        let request = Request::builder()
            .method("GET")
            .uri(&format!("/api/group_membership/group_chat/{}", group.id))
            .header("authorization", format!("Bearer {}", member_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found (user no longer member of group)
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Cleanup
        cleanup_test_user_from_a_group_chat(admin.id, group.id).await;
        cleanup_test_user_from_a_group_chat(member.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![admin.id, member.id]).await;
    }
}
