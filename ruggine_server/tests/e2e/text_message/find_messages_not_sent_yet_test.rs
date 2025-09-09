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
    cleanup_test_users_from_a_group_chat, cleanup_test_users, cleanup_test_user_from_a_group_chat,
    add_test_user_to_a_group, mark_message_as_sent, create_test_text_message, get_database
};
use chrono::{Duration, Utc};
use ruggine_server::utils::service_initializer::ServiceInitializer;
use ruggine_server::service::text_message_service::TextMessageServiceTrait;

// Helper function to mark a message as sent for a specific user
async fn mark_message_as_sent_for_user(message_id: i32, user_id: i32) {
    let db = get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    // Mark it as sent
    mark_message_as_sent(user_id, message_id).await;
}

#[cfg(test)]
mod find_messages_not_sent_yet_text_message_e2e_tests {
    use crate::{cleanup_text_message, common};
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_sent_yet_success_no_previous_sent() {
        // Arrange: Create router, users, group and messages
        let app = create_text_message_router().await;
        let (sender, _password, sender_token) = create_login_and_get_token("e2e_not_sent_sender".to_string()).await;
        let (reader, _password, reader_token) = create_login_and_get_token("e2e_not_sent_reader".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_not_sent_group", sender.id).await;

        // Create a message and mark it as sent, so there are no unsent messages
        let old_message = common::create_test_text_message(
            sender.id,
            group.id,
            Some("Old sent message should not be displayed by /group/{}/messages/not-sent-yet".to_string())
        ).await;

        let _reader_membership = add_test_user_to_a_group(reader.id, &group).await;

        // Create test messages that haven't been sent yet
        let message1 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("First message not sent".to_string())
        ).await;
        
        let message2 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Second message not sent".to_string())
        ).await;

        // Act: Send GET request to /group/{group_id}/messages/not-sent-yet with auth token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-sent-yet", group.id))
            .header("authorization", format!("Bearer {}", reader_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with unsent messages
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        assert!(response_json.get("pagination").is_some(), "Response should contain pagination field");
        
        let data = response_json["data"].as_array().unwrap();
        assert_eq!(data.len(), 2, "Should return both unsent messages");
        
        // Verify pagination
        let pagination = &response_json["pagination"];
        assert_eq!(pagination["has_more"].as_bool().unwrap(), true);
        assert_eq!(pagination["page_size"].as_i64().unwrap(), 2);
        
        // Verify message structure and data
        let message_ids: Vec<i64> = data.iter().map(|m| m["id"].as_i64().unwrap()).collect();
        assert!(message_ids.contains(&(message1.id as i64)));
        assert!(message_ids.contains(&(message2.id as i64)));

        for message_data in data {
            assert_eq!(message_data["group_chat_id"].as_i64().unwrap(), group.id as i64);
            assert!(message_data["id"].as_i64().is_some());
            assert!(message_data["content"].as_str().is_some());
            assert!(message_data["sender_id"].as_i64().is_some());
            assert!(message_data["sent_at"].as_str().is_some());
        }

        // Cleanup
        cleanup_text_messages(vec![old_message.id, message1.id, message2.id]).await;
        cleanup_test_user_from_a_group_chat(reader.id, group.id).await;
        cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(reader.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_sent_yet_success_with_previous_sent() {
        // Arrange: Create router, users, group and messages
        let app = create_text_message_router().await;
        let (sender, _password, sender_token) = create_login_and_get_token("e2e_sent_yet_sender".to_string()).await;
        let (reader, _password, reader_token) = create_login_and_get_token("e2e_sent_yet_reader".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_sent_yet_group", sender.id).await;
        let _reader_membership = add_test_user_to_a_group(reader.id, &group).await;

        // Create older message that was already sent
        let old_message = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Old sent message".to_string())
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        // Mark old message as sent
        mark_message_as_sent_for_user(old_message.id, reader.id).await;

        // Wait a bit and create new messages
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let new_message1 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("New message 1 not sent".to_string())
        ).await;

        let new_message2 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("New message 2 not sent".to_string())
        ).await;

        // Act: Send GET request to get unsent messages
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-sent-yet", group.id))
            .header("authorization", format!("Bearer {}", reader_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with only new unsent messages
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json["data"].as_array().unwrap();
        assert_eq!(data.len(), 2, "Should return only the new unsent messages");
        
        // Verify only new messages are returned, not the old sent one
        let message_ids: Vec<i64> = data.iter().map(|m| m["id"].as_i64().unwrap()).collect();
        assert!(message_ids.contains(&(new_message1.id as i64)));
        assert!(message_ids.contains(&(new_message2.id as i64)));
        assert!(!message_ids.contains(&(old_message.id as i64)));

        // Cleanup
        cleanup_text_messages(vec![old_message.id, new_message1.id, new_message2.id]).await;
        cleanup_test_user_from_a_group_chat(reader.id, group.id).await;
        cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(reader.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_sent_yet_empty_result() {
        // Arrange: Create router, users, group and messages
        let app = create_text_message_router().await;
        let (sender, _password, sender_token) = create_login_and_get_token("e2e_empty_result_sender".to_string()).await;
        let (reader, _password, reader_token) = create_login_and_get_token("e2e_empty_result_reader".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_empty_result_group", sender.id).await;
        let _reader_membership = add_test_user_to_a_group(reader.id, &group).await;

        // Create a message and mark it as sent, so there are no unsent messages
        let message = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Already sent message".to_string())
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        mark_message_as_sent_for_user(message.id, reader.id,).await;

        // Act: Send GET request to get unsent messages
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-sent-yet", group.id))
            .header("authorization", format!("Bearer {}", reader_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with empty result
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json["data"].as_array().unwrap();
        assert_eq!(data.len(), 0, "Should return empty result when all messages are sent");
        
        let pagination = &response_json["pagination"];
        assert_eq!(pagination["page_size"].as_i64().unwrap(), 0);

        // Cleanup
        cleanup_text_messages(vec![message.id]).await;
        cleanup_test_user_from_a_group_chat(reader.id, group.id).await;
        cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(reader.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_sent_yet_unauthorized_no_token() {
        // Arrange: Create router, user and group
        let app = create_text_message_router().await;
        let (user, _password, _token) = create_login_and_get_token("e2e_no_auth_user".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_no_auth_group", user.id).await;

        // Act: Send GET request without auth token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-sent-yet", group.id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_sent_yet_group_not_found() {
        // Arrange: Create router and user
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_group_not_found_user".to_string()).await;
        let non_existent_group_id = 99999;

        // Act: Send GET request to non-existent group
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-sent-yet", non_existent_group_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found or 403 Forbidden (depending on implementation)
        assert!(
            response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::FORBIDDEN,
            "Expected 404 or 403, got {}", response.status()
        );

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_sent_yet_user_not_member() {
        // Arrange: Create router, users and group
        let app = create_text_message_router().await;
        let (creator, _password, _creator_token) = create_login_and_get_token("e2e_not_member_creator".to_string()).await;
        let (non_member, _password, non_member_token) = create_login_and_get_token("e2e_not_member_user".to_string()).await;
        
        // Create group with creator only
        let group = create_test_group_chat_with_invitation_and_membership("e2e_not_member_group", creator.id).await;

        // Act: Try to access with user who is not a member
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-sent-yet", group.id))
            .header("authorization", format!("Bearer {}", non_member_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 403 Forbidden
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Cleanup
        cleanup_test_user_from_a_group_chat(creator.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(creator.email).await;
        cleanup_user_by_email(non_member.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_sent_yet_updates_sent_at() {
        // Arrange: Create router, users, group and message
        let app = create_text_message_router().await;
        let (sender, _password, sender_token) = create_login_and_get_token("e2e_update_sent_sender".to_string()).await;
        let (reader, _password, reader_token) = create_login_and_get_token("e2e_update_sent_reader".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_update_sent_group", sender.id).await;
        let _reader_membership = add_test_user_to_a_group(reader.id, &group).await;

        // Create test message that hasn't been sent yet
        let message = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Message to update sent_at".to_string())
        ).await;

        // Act: First call should return the message
        let request1 = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-sent-yet", group.id))
            .header("authorization", format!("Bearer {}", reader_token))
            .body(Body::empty())
            .unwrap();

        let response1 = app.clone().oneshot(request1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::OK);

        let body1 = to_bytes(response1.into_body(), usize::MAX).await.unwrap();
        let response_text1 = String::from_utf8(body1.to_vec()).unwrap();
        let response_json1: serde_json::Value = serde_json::from_str(&response_text1).unwrap();
        let data1 = response_json1["data"].as_array().unwrap();
        assert_eq!(data1.len(), 1, "Should return the unsent message");

        // Act: Second call should return empty result (message now marked as sent)
        let request2 = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-sent-yet", group.id))
            .header("authorization", format!("Bearer {}", reader_token))
            .body(Body::empty())
            .unwrap();

        let response2 = app.oneshot(request2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::OK);

        let body2 = to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let response_text2 = String::from_utf8(body2.to_vec()).unwrap();
        let response_json2: serde_json::Value = serde_json::from_str(&response_text2).unwrap();
        let data2 = response_json2["data"].as_array().unwrap();
        assert_eq!(data2.len(), 0, "Should return empty result as message is now marked as sent");

        // Cleanup
        cleanup_text_messages(vec![message.id]).await;
        cleanup_test_user_from_a_group_chat(reader.id, group.id).await;
        cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(reader.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_sent_yet_multiple_users_scenario() {
        // Arrange: Create router, users, group and message
        let app = create_text_message_router().await;
        let (sender, _password, sender_token) = create_login_and_get_token("e2e_multi_user_sender".to_string()).await;
        let (reader1, _password, reader1_token) = create_login_and_get_token("e2e_multi_user_reader1".to_string()).await;
        let (reader2, _password, reader2_token) = create_login_and_get_token("e2e_multi_user_reader2".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_multi_user_group", sender.id).await;
        let _reader1_membership = add_test_user_to_a_group(reader1.id, &group).await;
        let _reader2_membership = add_test_user_to_a_group(reader2.id, &group).await;

        // Create test message
        let message = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Message for multiple users".to_string())
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Mark as sent for reader1 only
        mark_message_as_sent_for_user(message.id, reader1.id,).await;

        // Act: Reader1 should get empty result (message already sent)
        let request1 = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-sent-yet", group.id))
            .header("authorization", format!("Bearer {}", reader1_token))
            .body(Body::empty())
            .unwrap();

        let response1 = app.clone().oneshot(request1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::OK);

        let body1 = to_bytes(response1.into_body(), usize::MAX).await.unwrap();
        let response_text1 = String::from_utf8(body1.to_vec()).unwrap();
        let response_json1: serde_json::Value = serde_json::from_str(&response_text1).unwrap();
        let data1 = response_json1["data"].as_array().unwrap();
        assert_eq!(data1.len(), 0, "Reader1 should get empty result");

        // Act: Reader2 should get the message (not sent yet)
        let request2 = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-sent-yet", group.id))
            .header("authorization", format!("Bearer {}", reader2_token))
            .body(Body::empty())
            .unwrap();

        let response2 = app.oneshot(request2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::OK);

        let body2 = to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let response_text2 = String::from_utf8(body2.to_vec()).unwrap();
        let response_json2: serde_json::Value = serde_json::from_str(&response_text2).unwrap();
        let data2 = response_json2["data"].as_array().unwrap();
        assert_eq!(data2.len(), 1, "Reader2 should get the unsent message");
        assert_eq!(data2[0]["id"].as_i64().unwrap(), message.id as i64);

        // Cleanup
        cleanup_text_messages(vec![message.id]).await;
        cleanup_test_user_from_a_group_chat(reader1.id, group.id).await;
        cleanup_test_user_from_a_group_chat(reader2.id, group.id).await;
        cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(reader1.email).await;
        cleanup_user_by_email(reader2.email).await;
    }
}
