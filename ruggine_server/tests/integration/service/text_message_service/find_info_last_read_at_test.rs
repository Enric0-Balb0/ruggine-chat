use ruggine_server::service::text_message_service::TextMessageServiceTrait;
use ruggine_server::factory::text_message_factory::TextMessageFactory;
use ruggine_server::dto::text_message_dto::TextMessageLastReadAtDto;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::group_membership_error::GroupMembershipError;
use crate::common;
use ruggine_server::utils::service_initializer::ServiceInitializer;
use chrono::Utc;
use ruggine_server::error::text_message_error::TextMessageError;

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_read_at_success() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_last_read_sender").await;
    let (reader_user, _) = common::create_test_user("find_last_read_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_last_read_group",
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

    // Mark messages as read at different times
    let read_time1 = Utc::now() - chrono::Duration::milliseconds(10);
    let read_time2 = Utc::now() - chrono::Duration::milliseconds(5);
    
    common::mark_message_as_sent_and_read(reader_user.id, message1.id, read_time1).await;
    common::mark_message_as_sent_and_read(reader_user.id, message2.id, read_time2).await;

    let payload = TextMessageLastReadAtDto {
        group_chat_id: group_chat.id,
    };

    // Act
    let result = service.find_info_last_read_at(reader_user.id, payload).await;

    // Assert - Should return the most recent read message (message2)
    assert!(result.is_ok(), "Failed to find last read message info: {:?}", result.err());
    let found_info = result.unwrap().unwrap();
    
    assert_eq!(found_info.text_message_id, message2.id);
    assert_eq!(found_info.user_id, reader_user.id);
    assert!(found_info.read_at.is_some());

    // Cleanup
    common::cleanup_text_message_info(info1.id).await;
    common::cleanup_text_message_info(info2.id).await;
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_read_at_no_read_messages() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_last_read_no_read_sender").await;
    let (reader_user, _) = common::create_test_user("find_last_read_no_read_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_last_read_no_read_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create test messages but don't mark them as read
    let message1 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Unread message 1".to_string())
    ).await;

    let message2 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Unread message 2".to_string())
    ).await;

    // Create text message infos for reader (but don't mark as read)
    let info1 = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(reader_user.id, message1.id),
        sender_user.id
    ).await.unwrap();

    let info2 = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(reader_user.id, message2.id),
        sender_user.id
    ).await.unwrap();

    let payload = TextMessageLastReadAtDto {
        group_chat_id: group_chat.id,
    };

    // Act
    let result = service.find_info_last_read_at(reader_user.id, payload).await;

    // Assert - Should return Ok with None since no messages are read
    assert!(result.is_ok(), "Should succeed even with no read messages: {:?}", result.err());

    // Cleanup
    common::cleanup_text_message_info(info1.id).await;
    common::cleanup_text_message_info(info2.id).await;
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_read_at_access_denied_not_member() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_last_read_access_sender").await;
    let (unauthorized_user, _) = common::create_test_user("find_last_read_access_unauthorized").await;
    
    // Create a test group with invitation and membership for the sender only
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_last_read_access_group",
        sender_user.id,
    ).await;

    let payload = TextMessageLastReadAtDto {
        group_chat_id: group_chat.id,
    };

    // Act - Try to access with unauthorized user
    let result = service.find_info_last_read_at(unauthorized_user.id, payload).await;

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
async fn test_find_info_last_read_at_access_denied_left_member() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_last_read_left_sender").await;
    let (left_user, _) = common::create_test_user("find_last_read_left_user").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_last_read_left_group",
        sender_user.id,
    ).await;

    // Add left_user to the group then remove them
    let _left_membership = common::add_test_user_to_a_group(left_user.id, &group_chat).await;

    // User leaves the group
    common::test_user_leave_from_a_group(left_user.id, group_chat.id).await;

    let payload = TextMessageLastReadAtDto {
        group_chat_id: group_chat.id,
    };

    // Act - Try to access with left user
    let result = service.find_info_last_read_at(left_user.id, payload).await;

    // Assert
    assert!(result.is_err(), "Expected access denied error for left member");
    
    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
            // Expected error type
        }
        _ => panic!("Expected UserCannotAccessMessages error"),
    }

    // Cleanup
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(left_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(left_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_read_at_mixed_read_unread() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_last_read_mixed_sender").await;
    let (reader_user, _) = common::create_test_user("find_last_read_mixed_reader").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_last_read_mixed_group",
        sender_user.id,
    ).await;

    // Add reader to the group
    let _reader_membership = common::add_test_user_to_a_group(reader_user.id, &group_chat).await;

    // Create test messages
    let message1 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("First message (read)".to_string())
    ).await;

    let message2 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Second message (unread)".to_string())
    ).await;

    let message3 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Third message (read latest)".to_string())
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

    // Mark only message1 and message3 as read, message2 remains unread
    let read_time1 = Utc::now() - chrono::Duration::milliseconds(10);
    let read_time3 = Utc::now() - chrono::Duration::milliseconds(2);
    
    common::mark_message_as_sent_and_read(reader_user.id, message1.id, read_time1).await;
    common::mark_message_as_sent_and_read(reader_user.id, message3.id, read_time3).await;
    // info2 (message2) is not marked as read

    let payload = TextMessageLastReadAtDto {
        group_chat_id: group_chat.id,
    };

    // Act
    let result = service.find_info_last_read_at(reader_user.id, payload).await;

    // Assert - Should return message3 (most recent read message)
    assert!(result.is_ok(), "Failed to find last read message info: {:?}", result.err());
    let found_info = result.unwrap().unwrap();
    
    assert_eq!(found_info.text_message_id, message3.id);
    assert_eq!(found_info.user_id, reader_user.id);
    assert!(found_info.read_at.is_some());

    // Cleanup
    common::cleanup_text_message_info(info1.id).await;
    common::cleanup_text_message_info(info2.id).await;
    common::cleanup_text_message_info(info3.id).await;
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_text_message(message3.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(reader_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(reader_user.id).await;
}
