use crate::common;
use chrono::Utc;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::text_message_error::TextMessageError;
use ruggine_server::service::text_message_service::TextMessageServiceTrait;
use ruggine_server::utils::service_initializer::ServiceInitializer;

#[tokio_shared_rt::test(shared)]
async fn test_find_first_message_with_no_read_at_success() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_first_no_read_sender").await;
    let (reader_user, _) = common::create_test_user("find_first_no_read_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_first_no_read_group",
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

    // Mark all messages as sent first (required before setting read_at)
    common::mark_message_as_sent(reader_user.id, message1.id).await;
    common::mark_message_as_sent(reader_user.id, message2.id).await;
    common::mark_message_as_sent(reader_user.id, message3.id).await;

    // Mark only message1 and message3 as read (leave message2 without read_at)
    common::mark_message_as_read(reader_user.id, message1.id).await;
    common::mark_message_as_read(reader_user.id, message3.id).await;

    // Act - Should find message2 as it's the first message without read_at
    let result = service.find_first_message_with_no_read_at(reader_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find first message with no read_at: {:?}", result.err());
    let found_message = result.unwrap();
    
    assert!(found_message.is_some(), "Expected to find a message without read_at");
    let message_dto = found_message.unwrap();
    assert_eq!(message_dto.id, message2.id);
    assert_eq!(message_dto.sender_id, sender_user.id);
    assert_eq!(message_dto.group_chat_id, group_chat.id);

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
async fn test_find_first_message_with_no_read_at_all_messages_read() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_first_all_read_sender").await;
    let (reader_user, _) = common::create_test_user("find_first_all_read_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_first_all_read_group",
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
    common::mark_message_as_sent_and_read(reader_user.id, message1.id).await;
    common::mark_message_as_sent_and_read(reader_user.id, message2.id).await;

    // Act
    let result = service.find_first_message_with_no_read_at(reader_user.id, group_chat.id).await;

    // Assert - Should return None as all messages have been read
    assert!(result.is_ok(), "Failed to execute find_first_message_with_no_read_at: {:?}", result.err());
    let found_message = result.unwrap();
    assert!(found_message.is_none(), "Expected no message without read_at");

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
async fn test_find_first_message_with_no_read_at_messages_sent_but_not_read() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_first_sent_not_read_sender").await;
    let (reader_user, _) = common::create_test_user("find_first_sent_not_read_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_first_sent_not_read_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create test messages
    let message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("First sent message".to_string())
    ).await;

    let message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Second sent message".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Mark all messages as sent but not read
    common::mark_message_as_sent(reader_user.id, message1.id).await;
    common::mark_message_as_sent(reader_user.id, message2.id).await;

    // Act - Should find the first message as none have been read
    let result = service.find_first_message_with_no_read_at(reader_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find first message with no read_at: {:?}", result.err());
    let found_message = result.unwrap();
    
    assert!(found_message.is_some(), "Expected to find a message without read_at");
    let message_dto = found_message.unwrap();
    assert_eq!(message_dto.id, message1.id, "Expected to find the first message");

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
async fn test_find_first_message_with_no_read_at_no_messages() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_first_no_read_no_msg_sender").await;
    let (reader_user, _) = common::create_test_user("find_first_no_read_no_msg_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_first_no_read_no_msg_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Act - No messages in the group
    let result = service.find_first_message_with_no_read_at(reader_user.id, group_chat.id).await;

    // Assert - Should return None as there are no messages
    assert!(result.is_ok(), "Failed to execute find_first_message_with_no_read_at: {:?}", result.err());
    let found_message = result.unwrap();
    assert!(found_message.is_none(), "Expected no message when group has no messages");

    // Cleanup
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_first_message_with_no_read_at_group_not_found() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (user, _) = common::create_test_user("find_first_no_read_group_not_found_user").await;
    let non_existing_group_id = 999999;

    // Act
    let result = service.find_first_message_with_no_read_at(user.id, non_existing_group_id).await;

    // Assert
    assert!(result.is_err(), "Expected error for non-existing group");
    
    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::GroupChatNotFound) => {
            // Expected error type
        }
        _ => panic!("Expected GroupChatNotFound error"),
    }

    // Cleanup
    common::cleanup_user(user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_first_message_with_no_read_at_user_not_member() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (group_owner, _) = common::create_test_user("find_first_no_read_not_member_owner").await;
    let (non_member_user, _) = common::create_test_user("find_first_no_read_not_member_user").await;
    
    // Create a test group with invitation and membership for the owner only
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_first_no_read_not_member_group",
        group_owner.id,
    ).await;

    // Act - User is not a member of the group
    let result = service.find_first_message_with_no_read_at(non_member_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_err(), "Expected error for non-member user");
    
    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
            // Expected error type
        }
        _ => panic!("Expected UserCannotAccessMessages error"),
    }

    // Cleanup
    common::cleanup_test_user_from_a_group_chat(group_owner.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(group_owner.id).await;
    common::cleanup_user(non_member_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_first_message_with_no_read_at_returns_oldest_unread() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_first_no_read_oldest_sender").await;
    let (reader_user, _) = common::create_test_user("find_first_no_read_oldest_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_first_no_read_oldest_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create test messages with delays to ensure different creation times
    let message1 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Oldest unread message".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let message2 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Middle unread message".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    let message3 = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Newest unread message".to_string())
    ).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Mark all messages as sent but not read
    common::mark_message_as_sent(reader_user.id, message1.id).await;
    common::mark_message_as_sent(reader_user.id, message2.id).await;
    common::mark_message_as_sent(reader_user.id, message3.id).await;

    // Act - Should find the oldest message (message1)
    let result = service.find_first_message_with_no_read_at(reader_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find first message with no read_at: {:?}", result.err());
    let found_message = result.unwrap();
    
    assert!(found_message.is_some(), "Expected to find a message without read_at");
    let message_dto = found_message.unwrap();
    assert_eq!(message_dto.id, message1.id, "Expected to find the oldest message first");
    assert_eq!(message_dto.content, "Oldest unread message");

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
