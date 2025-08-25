use std::ptr::read;
use ruggine_server::service::text_message_service::TextMessageServiceTrait;
use ruggine_server::factory::text_message_factory::TextMessageFactory;
use ruggine_server::dto::text_message_dto::TextMessageLastSentAtDto;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::group_membership_error::GroupMembershipError;
use crate::common;
use ruggine_server::utils::service_initializer::ServiceInitializer;
use chrono::Utc;
use ruggine_server::error::text_message_error::TextMessageError;

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_sent_at_success() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_last_sent_sender").await;
    let (reader_user, _) = common::create_test_user("find_last_sent_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_last_sent_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create test messages
    let message1 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("First message".to_string())
    ).await;

    let message2 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Second message".to_string())
    ).await;

    // Create text message infos for reader
    let info1 = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(reader_user.id, message1.id),
        sender_user.id
    ).await.unwrap();

    let info2 = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(reader_user.id, message2.id),
        sender_user.id
    ).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Mark messages as sent at different times
    let sent_time1 = Utc::now() - chrono::Duration::milliseconds(10);
    let sent_time2 = Utc::now() - chrono::Duration::milliseconds(5);
    
    common::mark_message_as_sent(reader_user.id, message1.id, sent_time1).await;
    common::mark_message_as_sent(reader_user.id, message2.id, sent_time2).await;

    let payload = TextMessageLastSentAtDto {
        group_chat_id: group_chat.id,
    };

    // Act
    let result = service.find_info_last_sent_at(reader_user.id, payload).await;

    // Assert - Should return the most recent sent message (message2)
    assert!(result.is_ok(), "Failed to find last sent message info: {:?}", result.err());
    let found_info = result.unwrap().unwrap();
    
    assert_eq!(found_info.text_message_id, message2.id);
    assert_eq!(found_info.user_id, reader_user.id);
    assert!(found_info.sent_at.is_some());

    // Cleanup
    common::cleanup_text_message_info(info1.id).await;
    common::cleanup_text_message_info(info2.id).await;
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_sent_at_no_sent_messages() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_last_sent_no_sent_sender").await;
    let (reader_user, _) = common::create_test_user("find_last_sent_no_sent_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_last_sent_no_sent_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create test messages but don't mark them as sent
    let message1 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Unsent message 1".to_string())
    ).await;

    let message2 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Unsent message 2".to_string())
    ).await;

    // Create text message infos for reader (but don't mark as sent)
    let info1 = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(reader_user.id, message1.id),
        sender_user.id
    ).await.unwrap();

    let info2 = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(reader_user.id, message2.id),
        sender_user.id
    ).await.unwrap();

    let payload = TextMessageLastSentAtDto {
        group_chat_id: group_chat.id,
    };

    // Act
    let result = service.find_info_last_sent_at(reader_user.id, payload).await;

    // Assert - Should return Ok with None since no messages are sent
    assert!(result.is_ok(), "Should succeed even with no sent messages: {:?}", result.err());

    // Cleanup
    common::cleanup_text_message_info(info1.id).await;
    common::cleanup_text_message_info(info2.id).await;
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_sent_at_access_denied_not_member() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_last_sent_access_sender").await;
    let (unauthorized_user, _) = common::create_test_user("find_last_sent_access_unauthorized").await;
    
    // Create a test group with invitation and membership for the sender only
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_last_sent_access_group",
        sender_user.id,
    ).await;

    let payload = TextMessageLastSentAtDto {
        group_chat_id: group_chat.id,
    };

    // Act - Try to access with unauthorized user
    let result = service.find_info_last_sent_at(unauthorized_user.id, payload).await;

    // Assert
    assert!(result.is_err(), "Expected access denied error");
    
    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
            // Expected error type
        }
        _ => panic!("Expected UserCannotAccessMessages error"),
    }

    // Cleanup
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(unauthorized_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_sent_at_access_denied_left_member() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_last_sent_left_sender").await;
    let (left_user, _) = common::create_test_user("find_last_sent_left_user").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_last_sent_left_group",
        sender_user.id,
    ).await;

    // Add left_user to the group then remove them
    let _left_membership = common::add_test_user_to_a_group(left_user.id, &group_chat).await;

    // User leaves the group
    common::test_user_leave_from_a_group(left_user.id, group_chat.id).await;

    let payload = TextMessageLastSentAtDto {
        group_chat_id: group_chat.id,
    };

    // Act - Try to access with left user
    let result = service.find_info_last_sent_at(left_user.id, payload).await;

    // Assert
    assert!(result.is_err(), "Expected access denied error for left member");
    
    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
            // Expected error type
        }
        _ => panic!("Expected UserCannotAccessMessages error"),
    }

    // Cleanup
    common::cleanup_test_user_from_a_group_chat(left_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(left_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_sent_at_mixed_sent_unsent() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_last_sent_mixed_sender").await;
    let (reader_user, _) = common::create_test_user("find_last_sent_mixed_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_last_sent_mixed_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create test messages
    let message1 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("First message (sent)".to_string())
    ).await;

    let message2 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Second message (unsent)".to_string())
    ).await;

    let message3 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Third message (sent latest)".to_string())
    ).await;

    // Create text message infos for reader
    let info1 = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(reader_user.id, message1.id),
        sender_user.id
    ).await.unwrap();

    let info2 = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(reader_user.id, message2.id),
        sender_user.id
    ).await.unwrap();

    let info3 = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(reader_user.id, message3.id),
        sender_user.id
    ).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Mark only message1 and message3 as sent, message2 remains unsent
    let sent_time1 = Utc::now() - chrono::Duration::milliseconds(10);
    let sent_time3 = Utc::now() - chrono::Duration::milliseconds(2);
    
    common::mark_message_as_sent(reader_user.id, message1.id, sent_time1).await;
    common::mark_message_as_sent(reader_user.id, message3.id, sent_time3).await;
    // info2 (message2) is not marked as sent

    let payload = TextMessageLastSentAtDto {
        group_chat_id: group_chat.id,
    };

    // Act
    let result = service.find_info_last_sent_at(reader_user.id, payload).await;

    // Assert - Should return message3 (most recent sent message)
    assert!(result.is_ok(), "Failed to find last sent message info: {:?}", result.err());
    let found_info = result.unwrap().unwrap();
    
    assert_eq!(found_info.text_message_id, message3.id);
    assert_eq!(found_info.user_id, reader_user.id);
    assert!(found_info.sent_at.is_some());

    // Cleanup
    common::cleanup_text_message_info(info1.id).await;
    common::cleanup_text_message_info(info2.id).await;
    common::cleanup_text_message_info(info3.id).await;
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
async fn test_find_info_last_sent_at_both_sent_and_read() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_last_sent_both_sender").await;
    let (reader_user, _) = common::create_test_user("find_last_sent_both_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_last_sent_both_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create test messages
    let message1 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("First message (sent and read)".to_string())
    ).await;

    let message2 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Second message (sent only)".to_string())
    ).await;

    // Create text message infos for reader
    let info1 = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(reader_user.id, message1.id),
        sender_user.id
    ).await.unwrap();

    let info2 = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(reader_user.id, message2.id),
        sender_user.id
    ).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Mark both messages as sent at different times
    let time1 = Utc::now() - chrono::Duration::milliseconds(10);
    let time2 = Utc::now() - chrono::Duration::milliseconds(5);
    
    // Message1: mark as both sent and read
    common::mark_message_as_sent_and_read(reader_user.id, message1.id, time1).await;
    
    // Message2: mark as sent only
    common::mark_message_as_sent(reader_user.id, message2.id, time2).await;

    let payload = TextMessageLastSentAtDto {
        group_chat_id: group_chat.id,
    };

    // Act
    let result = service.find_info_last_sent_at(reader_user.id, payload).await;

    // Assert - Should return message2 (most recent sent message)
    assert!(result.is_ok(), "Failed to find last sent message info: {:?}", result.err());
    let found_info = result.unwrap().unwrap();
    
    assert_eq!(found_info.text_message_id, message2.id);
    assert_eq!(found_info.user_id, reader_user.id);
    assert!(found_info.sent_at.is_some());
    // This message is sent but not read
    assert!(found_info.read_at.is_none());

    // Cleanup
    common::cleanup_text_message_info(info1.id).await;
    common::cleanup_text_message_info(info2.id).await;
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}
