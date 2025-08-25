use crate::common;
use chrono::Utc;
use ruggine_server::dto::group_membership_dto::LeaveGroupMembershipDto;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::text_message_error::TextMessageError;
use ruggine_server::service::text_message_service::TextMessageServiceTrait;
use ruggine_server::utils::service_initializer::ServiceInitializer;

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_read_yet_success() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_not_read_sender").await;
    let (reader_user, _) = common::create_test_user("find_not_read_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_not_read_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create test messages
    let message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("First message".to_string())
    ).await;

    let message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Second message".to_string())
    ).await;

    let message3 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Third message".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Mark only message1 as sent and read (message2 and message3 should be unread)
    let sent_time = Utc::now() - chrono::Duration::milliseconds(10);
    common::mark_message_as_sent_and_read(reader_user.id, message1.id, sent_time).await;

    // Act
    let result = service.find_messages_not_read_yet(group_chat.id, reader_user.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find messages not read yet: {:?}", result.err());
    let response = result.unwrap();
    
    // Should return message2 and message3 as they haven't been read yet
    assert_eq!(response.data.len(), 2, "Expected 2 unread messages");
    
    // Verify the messages are correctly identified
    let message_ids: Vec<i32> = response.data.iter().map(|m| m.id).collect();
    assert!(message_ids.contains(&message2.id), "Expected message2 to be in unread messages");
    assert!(message_ids.contains(&message3.id), "Expected message3 to be in unread messages");
    assert!(!message_ids.contains(&message1.id), "Message1 should not be in unread messages as it was already read");

    // Verify pagination metadata
    assert!(response.pagination.next_cursor.is_some(), "Expected next_cursor to be set");
    assert_eq!(response.pagination.page_size, 2, "Expected page_size to match returned data");

    // Cleanup
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_text_message(message3.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_read_yet_all_messages_read() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_all_read_sender").await;
    let (reader_user, _) = common::create_test_user("find_all_read_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_all_read_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create test messages
    let message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("First message".to_string())
    ).await;

    let message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Second message".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Mark all messages as sent and read
    let time = Utc::now() - chrono::Duration::milliseconds(10);
    common::mark_message_as_sent_and_read(reader_user.id, message1.id, time).await;
    common::mark_message_as_sent_and_read(reader_user.id, message2.id, time).await;

    // Act
    let result = service.find_messages_not_read_yet(group_chat.id, reader_user.id).await;

    // Assert - Should return empty list as all messages have been read
    assert!(result.is_ok(), "Failed to execute find_messages_not_read_yet: {:?}", result.err());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 0, "Expected no unread messages");
    assert!(response.pagination.next_cursor.is_none(), "Expected no next_cursor when no messages");

    // Cleanup
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_read_yet_no_messages() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_no_msg_sender").await;
    let (reader_user, _) = common::create_test_user("find_no_msg_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_no_msg_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Act - No messages in the group
    let result = service.find_messages_not_read_yet(group_chat.id, reader_user.id).await;

    // Assert - Should return empty list as there are no messages
    assert!(result.is_ok(), "Failed to execute find_messages_not_read_yet: {:?}", result.err());
    let response = result.unwrap();
    assert_eq!(response.data.len(), 0, "Expected no messages when group has no messages");
    assert!(response.pagination.next_cursor.is_none(), "Expected no next_cursor when no messages");

    // Cleanup
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_read_yet_mixed_sent_and_unsent() {
    // Arrange - Test scenario where some messages are sent but not read, others are not even sent
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_mixed_sender").await;
    let (reader_user, _) = common::create_test_user("find_mixed_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_mixed_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create test messages with delays to ensure different timestamps
    let message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("First message (will be read)".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Second message (sent but not read)".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let message3 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Third message (not sent)".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Mark message1 as sent and read
    let sent_read_time = Utc::now() - chrono::Duration::milliseconds(15);
    common::mark_message_as_sent_and_read(reader_user.id, message1.id, sent_read_time).await;

    // Mark message2 as sent but not read
    let sent_time = Utc::now() - chrono::Duration::milliseconds(10);
    common::mark_message_as_sent(reader_user.id, message2.id, sent_time).await;

    // Leave message3 without sent_at (not sent)

    // Act
    let result = service.find_messages_not_read_yet(group_chat.id, reader_user.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find messages not read yet: {:?}", result.err());
    let response = result.unwrap();
    
    // Should return both message2 and message3 as they need to be processed
    assert_eq!(response.data.len(), 2, "Expected 2 messages to be processed");
    
    // Verify the messages are correctly identified
    let message_ids: Vec<i32> = response.data.iter().map(|m| m.id).collect();
    assert!(message_ids.contains(&message2.id), "Expected message2 to be processed");
    assert!(message_ids.contains(&message3.id), "Expected message3 to be processed");
    assert!(!message_ids.contains(&message1.id), "Message1 should not be processed as it was already read");

    // Cleanup
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_text_message(message3.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_read_yet_updates_sent_and_read_status() {
    // Arrange - Test that the function properly updates sent_at and read_at timestamps
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_updates_sender").await;
    let (reader_user, _) = common::create_test_user("find_updates_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_updates_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create a test message that hasn't been sent or read
    let message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for status update".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Verify initial state - message should not be sent or read
    let info_before = service.find_info_by_user_id_and_message_id(reader_user.id, message.id).await.unwrap();
    assert!(info_before.sent_at.is_none(), "Message should not be sent initially");
    assert!(info_before.read_at.is_none(), "Message should not be read initially");

    // Act
    let result = service.find_messages_not_read_yet(group_chat.id, reader_user.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find messages not read yet: {:?}", result.err());
    let response = result.unwrap();
    
    assert_eq!(response.data.len(), 1, "Expected 1 message to be processed");
    assert_eq!(response.data[0].id, message.id, "Expected the correct message to be processed");

    // Verify that the message's status has been updated
    let info_after = service.find_info_by_user_id_and_message_id(reader_user.id, message.id).await.unwrap();
    assert!(info_after.sent_at.is_some(), "Message should be marked as sent after processing");
    assert!(info_after.read_at.is_none(), "Message should be marked not as read yet after processing");

    // Cleanup
    common::cleanup_text_message(message.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_read_yet_respects_datetime_range() {
    // Arrange - Test that the function uses the correct datetime range
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_range_sender").await;
    let (reader_user, _) = common::create_test_user("find_range_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_range_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create messages with different timestamps
    let old_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Old message".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let new_message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("New message 1".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let new_message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("New message 2".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Mark the old message as sent and read (this will be the baseline)
    let old_time = Utc::now() - chrono::Duration::milliseconds(20);
    common::mark_message_as_sent_and_read(reader_user.id, old_message.id, old_time).await;

    // Act
    let result = service.find_messages_not_read_yet(group_chat.id, reader_user.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find messages not read yet: {:?}", result.err());
    let response = result.unwrap();
    
    // Should only return the new messages, not the old one that was already processed
    assert_eq!(response.data.len(), 2, "Expected 2 new messages to be processed");
    
    let message_ids: Vec<i32> = response.data.iter().map(|m| m.id).collect();
    assert!(message_ids.contains(&new_message1.id), "Expected new_message1 to be processed");
    assert!(message_ids.contains(&new_message2.id), "Expected new_message2 to be processed");
    assert!(!message_ids.contains(&old_message.id), "Old message should not be processed again");

    // Cleanup
    common::cleanup_text_message(old_message.id).await;
    common::cleanup_text_message(new_message1.id).await;
    common::cleanup_text_message(new_message2.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_read_yet_user_not_member() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (group_owner, _) = common::create_test_user("find_not_read_owner").await;
    let (non_member_user, _) = common::create_test_user("find_not_read_non_member").await;
    
    // Create a test group with invitation and membership for the owner only
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_not_read_not_member_group",
        group_owner.id,
    ).await;

    // Create a message in the group
    let _message = common::create_test_text_message(
        group_owner.id, 
        group_chat.id, 
        Some("Test message".to_string())
    ).await;

    // Act - User is not a member of the group
    let result = service.find_messages_not_read_yet(group_chat.id, non_member_user.id).await;

    // Assert - Should fail because user is not a member
    assert!(result.is_err(), "Expected error for non-member user");
    
    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
            // Expected error type
        }
        _ => panic!("Expected UserCannotAccessMessages error"),
    }

    // Cleanup
    common::cleanup_text_message(_message.id).await;
    common::cleanup_test_user_from_a_group_chat(group_owner.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(group_owner.id).await;
    common::cleanup_user(non_member_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_messages_not_read_yet_user_left_and_rejoined_group() {
    // Arrange - Test scenario where user leaves group, messages are created, then user rejoins
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (group_owner, _) = common::create_test_user("rejoined_owner").await;
    let (reader_user, _) = common::create_test_user("rejoined_reader").await;
    let (sender_user, _) = common::create_test_user("rejoined_sender").await;
    
    // Create a test group with invitation and membership for the owner
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "rejoined_group",
        group_owner.id,
    ).await;

    // Add reader and sender to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;
    let _sender_membership = common::add_test_user_to_a_group(sender_user.id, &group_chat).await;

    // Phase 1: Create initial messages while reader is in the group
    let initial_message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Initial message 1".to_string())
    ).await;

    let initial_message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Initial message 2".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Reader reads the initial messages
    let read_time = Utc::now() - chrono::Duration::milliseconds(30);
    common::mark_message_as_sent_and_read(reader_user.id, initial_message1.id, read_time).await;
    common::mark_message_as_sent_and_read(reader_user.id, initial_message2.id, read_time).await;

    // Phase 2: Create some messages that reader doesn't read
    let unread_message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Unread message 1".to_string())
    ).await;

    let unread_message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Unread message 2".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Phase 3: Reader leaves the group
    common::test_user_leave_from_a_group(reader_user.id, group_chat.id).await;

    // Phase 4: Create messages while reader is NOT in the group (these won't have TextMessageInfo for reader)
    let missing_message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Message while user was absent 1".to_string())
    ).await;

    let missing_message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Message while user was absent 2".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Phase 5: Reader rejoins the group
    let _reader_membership_new = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Phase 6: Create more messages after rejoining
    let after_rejoin_message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Message after rejoin 1".to_string())
    ).await;

    let after_rejoin_message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Message after rejoin 2".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Act - Call find_messages_not_read_yet which should handle missing TextMessageInfo gracefully
    let result = service.find_messages_not_read_yet(group_chat.id, reader_user.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find messages not read yet: {:?}", result.err());
    let response = result.unwrap();
    
    // Should return all unread messages (both before leaving and after rejoining)
    // The messages created while user was absent should trigger MessageInfoNotFound errors 
    // but the function should handle them gracefully with warnings
    assert_eq!(response.data.len(), 6, "Expected at 6 unread messages (2 unread + 2 after left + 2 after rejoin), got: {}", response.data.len());
    
    // Verify that at least the messages we expect are included
    let message_ids: Vec<i32> = response.data.iter().map(|m| m.id).collect();
    
    // These should definitely be included
    assert!(message_ids.contains(&unread_message1.id), "Expected unread_message1 to be processed");
    assert!(message_ids.contains(&unread_message2.id), "Expected unread_message2 to be processed");
    assert!(message_ids.contains(&missing_message1.id), "Expected missing_message1 to be processed");
    assert!(message_ids.contains(&missing_message2.id), "Expected missing_message2 to be processed");
    assert!(message_ids.contains(&after_rejoin_message1.id), "Expected after_rejoin_message1 to be processed");
    assert!(message_ids.contains(&after_rejoin_message2.id), "Expected after_rejoin_message2 to be processed");
    
    // The initial messages should NOT be included as they were already read
    assert!(!message_ids.contains(&initial_message1.id), "Initial_message1 should not be processed as it was already read");
    assert!(!message_ids.contains(&initial_message2.id), "Initial_message2 should not be processed as it was already read");

    // Find a last time with no not read messages
    // let result = service.find_messages_not_read_yet(group_chat.id, reader_user.id).await;
    // assert_eq!(result.unwrap().data.len(), 0, "there should not be other not read messages");

    // Cleanup
    common::cleanup_text_message(initial_message1.id).await;
    common::cleanup_text_message(initial_message2.id).await;
    common::cleanup_text_message(missing_message1.id).await;
    common::cleanup_text_message(missing_message2.id).await;
    common::cleanup_text_message(unread_message1.id).await;
    common::cleanup_text_message(unread_message2.id).await;
    common::cleanup_text_message(after_rejoin_message1.id).await;
    common::cleanup_text_message(after_rejoin_message2.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(group_owner.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(group_owner.id).await;
    common::cleanup_user(reader_user.id).await;
    common::cleanup_user(sender_user.id).await;
}
