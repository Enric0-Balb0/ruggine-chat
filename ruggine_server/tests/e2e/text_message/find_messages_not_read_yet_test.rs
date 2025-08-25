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
    add_test_user_to_a_group, mark_message_as_sent_and_read, create_test_text_message, get_database,
    mark_message_as_sent, test_user_leave_from_a_group
};
use chrono::{Duration, Utc};
use ruggine_server::utils::service_initializer::ServiceInitializer;
use ruggine_server::service::text_message_service::TextMessageServiceTrait;

#[cfg(test)]
mod find_messages_not_read_yet_text_message_e2e_tests {
    use crate::{cleanup_text_message, common};
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_read_yet_success() {
        // Arrange: Create router, users, group and messages
        let app = create_text_message_router().await;
        let (sender, _password, sender_token) = create_login_and_get_token("e2e_not_read_sender".to_string()).await;
        let (reader, _password, reader_token) = create_login_and_get_token("e2e_not_read_reader".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_not_read_group", sender.id).await;
        let _reader_membership = add_test_user_to_a_group(reader.id, &group).await;

        // Create test messages
        let message1 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("First message".to_string())
        ).await;

        let message2 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Second message".to_string())
        ).await;

        let message3 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Third message".to_string())
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Mark only message1 as sent and read (message2 and message3 should be unread)
        let sent_time = Utc::now() - chrono::Duration::milliseconds(10);
        mark_message_as_sent_and_read(reader.id, message1.id, sent_time).await;

        // Act: Send GET request to /group/{group_id}/messages/not-read-yet with auth token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-read-yet", group.id))
            .header("authorization", format!("Bearer {}", reader_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with unread messages
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        assert!(response_json.get("pagination").is_some(), "Response should contain pagination field");
        
        let data = response_json["data"].as_array().unwrap();
        assert_eq!(data.len(), 2, "Should return 2 unread messages");
        
        // Verify pagination
        let pagination = &response_json["pagination"];
        assert_eq!(pagination["has_more"].as_bool().unwrap(), true);
        assert_eq!(pagination["page_size"].as_i64().unwrap(), 2);
        
        // Verify message structure and data
        let message_ids: Vec<i64> = data.iter().map(|m| m["id"].as_i64().unwrap()).collect();
        assert!(message_ids.contains(&(message2.id as i64)));
        assert!(message_ids.contains(&(message3.id as i64)));
        assert!(!message_ids.contains(&(message1.id as i64)), "Message1 should not be in unread messages as it was already read");

        for message_data in data {
            assert_eq!(message_data["group_chat_id"].as_i64().unwrap(), group.id as i64);
            assert_eq!(message_data["sender_id"].as_i64().unwrap(), sender.id as i64);
            assert!(message_data["id"].as_i64().is_some());
            assert!(message_data["content"].as_str().is_some());
            assert!(message_data["sent_at"].as_str().is_some());
        }

        // Cleanup
        cleanup_text_message(message1.id).await;
        cleanup_text_message(message2.id).await;
        cleanup_text_message(message3.id).await;
        cleanup_test_user_from_a_group_chat(reader.id, group.id).await;
        cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(reader.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_read_yet_all_messages_read() {
        // Arrange: Create router, users, group and messages
        let app = create_text_message_router().await;
        let (sender, _password, sender_token) = create_login_and_get_token("e2e_all_read_sender".to_string()).await;
        let (reader, _password, reader_token) = create_login_and_get_token("e2e_all_read_reader".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_all_read_group", sender.id).await;
        let _reader_membership = add_test_user_to_a_group(reader.id, &group).await;

        // Create test messages
        let message1 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("First message".to_string())
        ).await;

        let message2 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Second message".to_string())
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Mark all messages as sent and read
        let time = Utc::now() - chrono::Duration::milliseconds(10);
        mark_message_as_sent_and_read(reader.id, message1.id, time).await;
        mark_message_as_sent_and_read(reader.id, message2.id, time).await;

        // Act: Send GET request to /group/{group_id}/messages/not-read-yet
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-read-yet", group.id))
            .header("authorization", format!("Bearer {}", reader_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with empty list
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json["data"].as_array().unwrap();
        assert_eq!(data.len(), 0, "Should return no unread messages");

        let pagination = &response_json["pagination"];
        assert!(pagination["next_cursor"].is_null(), "Expected no next_cursor when no messages");

        // Cleanup
        cleanup_text_message(message1.id).await;
        cleanup_text_message(message2.id).await;
        cleanup_test_user_from_a_group_chat(reader.id, group.id).await;
        cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(reader.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_read_yet_no_messages() {
        // Arrange: Create router, users and group without messages
        let app = create_text_message_router().await;
        let (sender, _password, sender_token) = create_login_and_get_token("e2e_no_msg_sender".to_string()).await;
        let (reader, _password, reader_token) = create_login_and_get_token("e2e_no_msg_reader".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_no_msg_group", sender.id).await;
        let _reader_membership = add_test_user_to_a_group(reader.id, &group).await;

        // Act: Send GET request to /group/{group_id}/messages/not-read-yet with no messages in group
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-read-yet", group.id))
            .header("authorization", format!("Bearer {}", reader_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with empty list
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json["data"].as_array().unwrap();
        assert_eq!(data.len(), 0, "Should return no messages when group has no messages");

        let pagination = &response_json["pagination"];
        assert!(pagination["next_cursor"].is_null(), "Expected no next_cursor when no messages");

        // Cleanup
        cleanup_test_user_from_a_group_chat(reader.id, group.id).await;
        cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(reader.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_read_yet_mixed_sent_and_unsent() {
        // Arrange: Test scenario where some messages are sent but not read, others are not even sent
        let app = create_text_message_router().await;
        let (sender, _password, sender_token) = create_login_and_get_token("e2e_mixed_sender".to_string()).await;
        let (reader, _password, reader_token) = create_login_and_get_token("e2e_mixed_reader".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_mixed_group", sender.id).await;
        let _reader_membership = add_test_user_to_a_group(reader.id, &group).await;

        // Create test messages with delays to ensure different timestamps
        let message1 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("First message (will be read)".to_string())
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let message2 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Second message (sent but not read)".to_string())
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let message3 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Third message (not sent)".to_string())
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Mark message1 as sent and read
        let sent_read_time = Utc::now() - chrono::Duration::milliseconds(15);
        mark_message_as_sent_and_read(reader.id, message1.id, sent_read_time).await;

        // Mark message2 as sent but not read
        let sent_time = Utc::now() - chrono::Duration::milliseconds(10);
        mark_message_as_sent(reader.id, message2.id, sent_time).await;

        // Leave message3 without sent_at (not sent)

        // Act: Send GET request
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-read-yet", group.id))
            .header("authorization", format!("Bearer {}", reader_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with both message2 and message3
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json["data"].as_array().unwrap();
        assert_eq!(data.len(), 2, "Should return 2 messages to be processed");
        
        // Verify the messages are correctly identified
        let message_ids: Vec<i64> = data.iter().map(|m| m["id"].as_i64().unwrap()).collect();
        assert!(message_ids.contains(&(message2.id as i64)), "Expected message2 to be processed");
        assert!(message_ids.contains(&(message3.id as i64)), "Expected message3 to be processed");
        assert!(!message_ids.contains(&(message1.id as i64)), "Message1 should not be processed as it was already read");

        // Cleanup
        cleanup_text_message(message1.id).await;
        cleanup_text_message(message2.id).await;
        cleanup_text_message(message3.id).await;
        cleanup_test_user_from_a_group_chat(reader.id, group.id).await;
        cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(reader.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_read_yet_unauthorized_no_token() {
        // Arrange: Create router, user and group with messages
        let app = create_text_message_router().await;
        let (user, _password, _token) = create_login_and_get_token("e2e_not_read_no_auth".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_not_read_no_auth", user.id).await;
        let message = create_test_text_message(user.id, group.id, Some("Test message".to_string())).await;

        // Act: Send GET request without auth token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-read-yet", group.id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        cleanup_text_message(message.id).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_read_yet_unauthorized_invalid_token() {
        // Arrange: Create router, user and group
        let app = create_text_message_router().await;
        let (user, _password, _token) = create_login_and_get_token("e2e_not_read_invalid_auth".to_string()).await;
        let group = create_test_group_chat_with_invitation_and_membership("e2e_not_read_invalid_auth", user.id).await;
        let message = create_test_text_message(user.id, group.id, Some("Test message".to_string())).await;

        // Act: Send GET request with invalid auth token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-read-yet", group.id))
            .header("authorization", "Bearer invalid_token_here")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        cleanup_text_message(message.id).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_read_yet_user_not_member() {
        // Arrange: Create users - one who is NOT member of the group
        let app = create_text_message_router().await;
        let (group_owner, _password1, _token1) = create_login_and_get_token("e2e_not_read_owner".to_string()).await;
        let (unauthorized_user, _password2, unauthorized_token) = create_login_and_get_token("e2e_not_read_unauthorized".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_not_read_unauthorized_group", group_owner.id).await;
        let message = create_test_text_message(group_owner.id, group.id, Some("Test message".to_string())).await;

        // Act: Send GET request with unauthorized user's token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-read-yet", group.id))
            .header("authorization", format!("Bearer {}", unauthorized_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 403 Forbidden (user cannot access messages)
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // Cleanup
        cleanup_text_message(message.id).await;
        cleanup_test_user_from_a_group_chat(group_owner.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(group_owner.email).await;
        cleanup_user_by_email(unauthorized_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_read_yet_user_left_and_rejoined_group() {
        // Arrange: Test scenario where user leaves group, messages are created, then user rejoins
        let app = create_text_message_router().await;
        let (group_owner, _password1, _token1) = create_login_and_get_token("e2e_rejoined_owner".to_string()).await;
        let (reader, _password2, reader_token) = create_login_and_get_token("e2e_rejoined_reader".to_string()).await;
        let (sender, _password3, _token3) = create_login_and_get_token("e2e_rejoined_sender".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_rejoined_group", group_owner.id).await;
        let _reader_membership = add_test_user_to_a_group(reader.id, &group).await;
        let _sender_membership = add_test_user_to_a_group(sender.id, &group).await;

        // Phase 1: Create initial messages while reader is in the group
        let initial_message1 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Initial message 1".to_string())
        ).await;

        let initial_message2 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Initial message 2".to_string())
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Reader reads the initial messages
        let read_time = Utc::now() - chrono::Duration::milliseconds(30);
        mark_message_as_sent_and_read(reader.id, initial_message1.id, read_time).await;
        mark_message_as_sent_and_read(reader.id, initial_message2.id, read_time).await;

        // Phase 2: Create some messages that reader doesn't read
        let unread_message1 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Unread message 1".to_string())
        ).await;

        let unread_message2 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Unread message 2".to_string())
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Phase 3: Reader leaves the group
        test_user_leave_from_a_group(reader.id, group.id).await;

        // Phase 4: Create messages while reader is NOT in the group
        let missing_message1 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Message while user was absent 1".to_string())
        ).await;

        let missing_message2 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Message while user was absent 2".to_string())
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Phase 5: Reader rejoins the group
        let _reader_membership_new = add_test_user_to_a_group(reader.id, &group).await;

        // Phase 6: Create more messages after rejoining
        let after_rejoin_message1 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Message after rejoin 1".to_string())
        ).await;

        let after_rejoin_message2 = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Message after rejoin 2".to_string())
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Act: Send GET request to find unread messages
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-read-yet", group.id))
            .header("authorization", format!("Bearer {}", reader_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with all unread messages
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json["data"].as_array().unwrap();
        // Should return all unread messages (2 unread + 2 missing + 2 after rejoin)
        assert_eq!(data.len(), 6, "Expected 6 unread messages, got: {}", data.len());
        
        // Verify that the expected messages are included
        let message_ids: Vec<i64> = data.iter().map(|m| m["id"].as_i64().unwrap()).collect();
        
        // These should definitely be included
        assert!(message_ids.contains(&(unread_message1.id as i64)), "Expected unread_message1 to be processed");
        assert!(message_ids.contains(&(unread_message2.id as i64)), "Expected unread_message2 to be processed");
        assert!(message_ids.contains(&(missing_message1.id as i64)), "Expected missing_message1 to be processed");
        assert!(message_ids.contains(&(missing_message2.id as i64)), "Expected missing_message2 to be processed");
        assert!(message_ids.contains(&(after_rejoin_message1.id as i64)), "Expected after_rejoin_message1 to be processed");
        assert!(message_ids.contains(&(after_rejoin_message2.id as i64)), "Expected after_rejoin_message2 to be processed");
        
        // The initial messages should NOT be included as they were already read
        assert!(!message_ids.contains(&(initial_message1.id as i64)), "Initial_message1 should not be processed as it was already read");
        assert!(!message_ids.contains(&(initial_message2.id as i64)), "Initial_message2 should not be processed as it was already read");

        // Cleanup
        cleanup_text_message(initial_message1.id).await;
        cleanup_text_message(initial_message2.id).await;
        cleanup_text_message(missing_message1.id).await;
        cleanup_text_message(missing_message2.id).await;
        cleanup_text_message(unread_message1.id).await;
        cleanup_text_message(unread_message2.id).await;
        cleanup_text_message(after_rejoin_message1.id).await;
        cleanup_text_message(after_rejoin_message2.id).await;
        cleanup_test_user_from_a_group_chat(reader.id, group.id).await;
        cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
        cleanup_test_user_from_a_group_chat(group_owner.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(group_owner.email).await;
        cleanup_user_by_email(reader.email).await;
        cleanup_user_by_email(sender.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_read_yet_updates_sent_and_read_status() {
        // Arrange: Test that the API properly updates sent_at and read_at timestamps
        let app = create_text_message_router().await;
        let (sender, _password, sender_token) = create_login_and_get_token("e2e_updates_sender".to_string()).await;
        let (reader, _password, reader_token) = create_login_and_get_token("e2e_updates_reader".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_updates_group", sender.id).await;
        let _reader_membership = add_test_user_to_a_group(reader.id, &group).await;

        // Create a test message that hasn't been sent or read
        let message = create_test_text_message(
            sender.id, 
            group.id, 
            Some("Test message for status update".to_string())
        ).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Verify initial state - message should not be sent or read
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let service = service_init.text_message_service();
        let info_before = service.find_info_by_user_id_and_message_id(reader.id, message.id).await.unwrap();
        assert!(info_before.sent_at.is_none(), "Message should not be sent initially");
        assert!(info_before.read_at.is_none(), "Message should not be read initially");

        // Act: Send GET request to find unread messages
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-read-yet", group.id))
            .header("authorization", format!("Bearer {}", reader_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK and update message status
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json["data"].as_array().unwrap();
        assert_eq!(data.len(), 1, "Expected 1 message to be processed");
        assert_eq!(data[0]["id"].as_i64().unwrap(), message.id as i64, "Expected the correct message to be processed");

        // Verify that the message's status has been updated
        let info_after = service.find_info_by_user_id_and_message_id(reader.id, message.id).await.unwrap();
        assert!(info_after.sent_at.is_some(), "Message should be marked as sent after processing");
        assert!(info_after.read_at.is_none(), "Message should be marked not as read yet after processing");

        // Cleanup
        cleanup_text_message(message.id).await;
        cleanup_test_user_from_a_group_chat(reader.id, group.id).await;
        cleanup_test_user_from_a_group_chat(sender.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user_by_email(sender.email).await;
        cleanup_user_by_email(reader.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_messages_not_read_yet_group_not_found() {
        // Arrange: Create router and user
        let app = create_text_message_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_not_read_group_not_found".to_string()).await;
        let non_existing_group_id = 999999;

        // Act: Send GET request with non-existing group ID
        let request = Request::builder()
            .method("GET")
            .uri(format!("/group/{}/messages/not-read-yet", non_existing_group_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found (group not found)
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }
}
