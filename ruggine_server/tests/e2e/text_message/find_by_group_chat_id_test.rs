use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use axum::body::to_bytes;
use crate::common::{
    cleanup_user_by_email, cleanup_group_chat, cleanup_text_messages,
    create_text_message_router, create_login_and_get_token,
    create_test_group_chat_with_invitation_and_membership,
    create_test_text_messages_for_group_without_message_info, create_test_users_for_a_group,
    create_test_text_messages_multi_sender_without_message_info, cleanup_test_users_from_a_group_chat,
    cleanup_test_users, cleanup_test_user_from_a_group_chat
};

#[cfg(test)]
mod find_by_group_chat_id_text_message_e2e_tests {
    use crate::test_user_leave_from_a_group;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_success() {
        // Arrange: Create router, user, group and messages
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_text_msg_success".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_text_msg_success", user.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, user.id, 5).await;

        // Act: Send GET request to /{group_id}/messages with auth token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages", group.id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with paginated messages
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        assert!(response_json.get("pagination").is_some(), "Response should contain pagination field");
        
        let data = response_json["data"].as_array().unwrap();
        assert_eq!(data.len(), 5, "Should return all 5 messages");
        
        // Verify pagination
        let pagination = &response_json["pagination"];
        assert_eq!(pagination["has_more"].as_bool().unwrap(), false);
        assert!(pagination["next_cursor"].is_null());
        assert_eq!(pagination["page_size"].as_i64().unwrap(), 5);
        
        // Verify message structure and data
        for message_data in data {
            assert_eq!(message_data["group_chat_id"].as_i64().unwrap(), group.id as i64);
            assert_eq!(message_data["sender_id"].as_i64().unwrap(), user.id as i64);
            assert!(message_data["id"].as_i64().is_some());
            assert!(message_data["content"].as_str().is_some());
            assert!(message_data["sent_at"].as_str().is_some());
        }

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_unauthorized_no_token() {
        // Arrange: Create router, user and group with messages
        let app = create_text_message_router().await;
        let (user, _password, _token) = create_login_and_get_token("e2e_text_msg_no_auth".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_text_msg_no_auth", user.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, user.id, 3).await;

        // Act: Send GET request without auth token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages", group.id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_unauthorized_invalid_token() {
        // Arrange: Create router, user and group
        let app = create_text_message_router().await;
        let (user, _password, _token) = create_login_and_get_token("e2e_text_msg_invalid_auth".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_text_msg_invalid_auth", user.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, user.id, 3).await;

        // Act: Send GET request with invalid auth token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages", group.id))
            .header("authorization", "Bearer invalid_token_here")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_user_not_member() {
        // Arrange: Create users - one who is NOT member of the group
        let app = create_text_message_router().await;
        let (group_owner, _password1, _token1) = create_login_and_get_token("e2e_text_msg_owner".to_string()).await;
        let (unauthorized_user, _password2, unauthorized_token) = create_login_and_get_token("e2e_text_msg_unauthorized".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_text_msg_unauthorized_group", group_owner.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, group_owner.id, 3).await;

        // Act: Send GET request with unauthorized user's token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages", group.id))
            .header("authorization", format!("Bearer {}", unauthorized_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 403 Forbidden or appropriate error status
        // The exact status depends on the implementation, but it should not be 200
        assert_eq!(response.status(), StatusCode::FORBIDDEN,
                   "Should return error status for non-member user, got: {}", response.status());

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_user_from_a_group_chat(group_owner.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(group_owner.email).await;
        cleanup_user_by_email(unauthorized_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_empty_group() {
        // Arrange: Create router, user and empty group
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_text_msg_empty".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_text_msg_empty", user.id).await;

        // Act: Send GET request to empty group
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages", group.id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with empty array
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json["data"].as_array().unwrap();
        assert_eq!(data.len(), 0, "Should return empty array for group with no messages");
        
        let pagination = &response_json["pagination"];
        assert_eq!(pagination["has_more"].as_bool().unwrap(), false);
        assert!(pagination["next_cursor"].is_null());
        assert_eq!(pagination["page_size"].as_i64().unwrap(), 0);

        // Cleanup
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_with_pagination_limit() {
        // Arrange: Create router, user, group and many messages
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_text_msg_paginated".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_text_msg_paginated", user.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, user.id, 15).await;

        // Act: Send GET request with limit parameter
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages?limit=10", group.id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with limited results
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json["data"].as_array().unwrap();
        assert_eq!(data.len(), 10, "Should return exactly 10 messages due to limit");
        
        let pagination = &response_json["pagination"];
        assert_eq!(pagination["has_more"].as_bool().unwrap(), true);
        assert!(pagination["next_cursor"].as_str().is_some(), "Should have next cursor");
        assert_eq!(pagination["page_size"].as_i64().unwrap(), 10);

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_with_cursor_pagination() {
        // Arrange: Create router, user, group and messages
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_text_msg_cursor".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_text_msg_cursor", user.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, user.id, 10).await;

        // First, get first page to obtain cursor
        let first_request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages?limit=5", group.id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let first_response = app.oneshot(first_request).await.unwrap();
        assert_eq!(first_response.status(), StatusCode::OK);

        let first_body = to_bytes(first_response.into_body(), usize::MAX).await.unwrap();
        let first_response_text = String::from_utf8(first_body.to_vec()).unwrap();
        let first_response_json: serde_json::Value = serde_json::from_str(&first_response_text).unwrap();

        let cursor = first_response_json["pagination"]["next_cursor"].as_str().unwrap();

        // Act: Send GET request with cursor parameter
        let app2 = create_text_message_router().await; // Create new app instance for second request
        let second_request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages?limit=5&cursor={}", group.id, cursor))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let second_response = app2.oneshot(second_request).await.unwrap();

        // Assert: Should return 200 OK with remaining messages
        assert_eq!(second_response.status(), StatusCode::OK);

        let second_body = to_bytes(second_response.into_body(), usize::MAX).await.unwrap();
        let second_response_text = String::from_utf8(second_body.to_vec()).unwrap();
        let second_response_json: serde_json::Value = serde_json::from_str(&second_response_text).unwrap();

        let second_data = second_response_json["data"].as_array().unwrap();
        assert_eq!(second_data.len(), 5, "Should return remaining 5 messages");

        let second_pagination = &second_response_json["pagination"];
        assert_eq!(second_pagination["has_more"].as_bool().unwrap(), false, "Should not have more pages");

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_multi_sender() {
        // Arrange: Create router, multiple users, group and messages from different senders
        let app = create_text_message_router().await;
        let (creator_user, _password1, token) = create_login_and_get_token("e2e_text_msg_multi_creator".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_text_msg_multi_group", creator_user.id).await;
        
        let users = create_test_users_for_a_group("e2e_text_msg_multi", 3, &group).await;
        let sender_ids: Vec<i32> = users.iter().map(|(user, _, _)| user.id).collect();
        let messages = create_test_text_messages_multi_sender_without_message_info(group.id, sender_ids.clone()).await;

        // Act: Send GET request as creator (who has access)
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages", group.id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with messages from all senders
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json["data"].as_array().unwrap();
        assert_eq!(data.len(), 3, "Should return messages from all senders");

        // Verify all sender IDs are present
        let found_sender_ids: std::collections::HashSet<i64> = data.iter()
            .map(|msg| msg["sender_id"].as_i64().unwrap())
            .collect();
        let expected_sender_ids: std::collections::HashSet<i64> = sender_ids.iter()
            .map(|&id| id as i64)
            .collect();
        assert_eq!(found_sender_ids, expected_sender_ids, "Should have messages from all expected senders");

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_users_from_a_group_chat(sender_ids.clone(), group.id).await;
        cleanup_test_user_from_a_group_chat(creator_user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(sender_ids).await;
        cleanup_user_by_email(creator_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_non_existing_group() {
        // Arrange: Create router and user
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_text_msg_nonexisting".to_string()).await;
        let non_existing_group_id = 999999;

        // Act: Send GET request to non-existing group
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages", non_existing_group_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return error status (not 200)
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "Should return error status for non-existing group, got: {}", response.status());

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_user_left_group() {
        // Arrange: Create router, user, group and messages
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_text_msg_left_group".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_text_msg_left_group", user.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, user.id, 3).await;

        // Act: User leaves the group first
        test_user_leave_from_a_group(user.id, group.id).await;

        // Now try to access messages after leaving
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages", group.id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 403 Forbidden since user left the group
        assert_eq!(response.status(), StatusCode::FORBIDDEN, "Should return forbidden for user who left group");

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }
}
