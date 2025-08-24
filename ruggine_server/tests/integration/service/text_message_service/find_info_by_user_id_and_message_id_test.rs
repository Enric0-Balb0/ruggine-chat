use ruggine_server::service::text_message_service::TextMessageServiceTrait;
use ruggine_server::factory::text_message_factory::TextMessageFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::text_message_error::TextMessageError;
use ruggine_server::error::group_membership_error::GroupMembershipError;
use crate::common;
use ruggine_server::utils::service_initializer::ServiceInitializer;

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_user_id_and_message_id_success() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_info_user_msg_sender").await;
    let (recipient_user, _) = common::create_test_user("find_info_user_msg_recipient").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_info_user_msg_group",
        sender_user.id,
    ).await;

    // Add recipient to the group
    let _recipient_membership = common::add_test_user_to_a_group(recipient_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for find info by user and message".to_string())
    ).await;

    // Create text message info for both users
    let sender_create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let sender_info = service.create_info(sender_create_payload, sender_user.id).await.unwrap();

    let recipient_create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        recipient_user.id, 
        text_message.id
    );
    let recipient_info = service.create_info(recipient_create_payload, sender_user.id).await.unwrap();

    // Act - Find sender's info
    let sender_result = service.find_info_by_user_id_and_message_id(sender_user.id, text_message.id).await;

    // Assert for sender
    assert!(sender_result.is_ok(), "Failed to find sender's text message info: {:?}", sender_result.err());
    let found_sender_info = sender_result.unwrap();
    
    assert_eq!(found_sender_info.id, sender_info.id);
    assert_eq!(found_sender_info.user_id, sender_user.id);
    assert_eq!(found_sender_info.text_message_id, text_message.id);

    // Act - Find recipient's info
    let recipient_result = service.find_info_by_user_id_and_message_id(recipient_user.id, text_message.id).await;

    // Assert for recipient
    assert!(recipient_result.is_ok(), "Failed to find recipient's text message info: {:?}", recipient_result.err());
    let found_recipient_info = recipient_result.unwrap();
    
    assert_eq!(found_recipient_info.id, recipient_info.id);
    assert_eq!(found_recipient_info.user_id, recipient_user.id);
    assert_eq!(found_recipient_info.text_message_id, text_message.id);

    // Cleanup
    common::cleanup_text_message_info(sender_info.id).await;
    common::cleanup_text_message_info(recipient_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(recipient_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_user_id_and_message_id_not_found() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_info_not_found_sender").await;
    let (other_user, _) = common::create_test_user("find_info_not_found_other").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_info_not_found_group",
        sender_user.id,
    ).await;

    // Add other user to the group
    let _other_membership = common::add_test_user_to_a_group(other_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message with no info for other user".to_string())
    ).await;

    // Create text message info for sender only
    let sender_create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let sender_info = service.create_info(sender_create_payload, sender_user.id).await.unwrap();

    // Act - Try to find info for other_user (who has no info for this message)
    let result = service.find_info_by_user_id_and_message_id(other_user.id, text_message.id).await;

    // Assert
    assert!(result.is_err(), "Expected error for non-existing message info");
    
    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::MessageInfoNotFound) => {
            // Expected error type
        }
        _ => panic!("Expected MessageInfoNotFound error"),
    }

    // Cleanup
    common::cleanup_text_message_info(sender_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(other_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(other_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_user_id_and_message_id_access_denied_not_member() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_info_access_sender").await;
    let (unauthorized_user, _) = common::create_test_user("find_info_access_unauthorized").await;
    
    // Create a test group with invitation and membership for the sender only
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_info_access_group",
        sender_user.id,
    ).await;

    // Create a test message
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for access denied".to_string())
    ).await;

    // Act - Try to access with unauthorized user
    let result = service.find_info_by_user_id_and_message_id(unauthorized_user.id, text_message.id).await;

    // Assert
    assert!(result.is_err(), "Expected access denied error");

    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
            // Expected error type
        }
        _ => panic!("Expected UserCannotAccessMessages error"),
    }

    // Cleanup
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(unauthorized_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_user_id_and_message_id_message_not_found() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (user, _) = common::create_test_user("find_info_msg_not_found_user").await;
    
    let non_existing_message_id = 999999;

    // Act
    let result = service.find_info_by_user_id_and_message_id(user.id, non_existing_message_id).await;

    // Assert
    assert!(result.is_err(), "Expected error for non-existing message");
    
    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::MessageNotFound) => {
            // Expected error type
        }
        _ => panic!("Expected MessageNotFound error"),
    }

    // Cleanup
    common::cleanup_user(user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_user_id_and_message_id_multiple_messages() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_info_multi_sender").await;
    let (recipient_user, _) = common::create_test_user("find_info_multi_recipient").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_info_multi_group",
        sender_user.id,
    ).await;

    // Add recipient to the group
    let _recipient_membership = common::add_test_user_to_a_group(recipient_user.id, &group_chat).await;

    // Create multiple test messages
    let message1 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("First test message".to_string())
    ).await;

    let message2 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Second test message".to_string())
    ).await;

    // Create text message info for recipient for both messages
    let msg1_info = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(recipient_user.id, message1.id),
        sender_user.id
    ).await.unwrap();

    let msg2_info = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(recipient_user.id, message2.id),
        sender_user.id
    ).await.unwrap();

    // Act - Find info for recipient for message1
    let result1 = service.find_info_by_user_id_and_message_id(recipient_user.id, message1.id).await;

    // Assert for message1
    assert!(result1.is_ok(), "Failed to find info for message1: {:?}", result1.err());
    let found_info1 = result1.unwrap();
    assert_eq!(found_info1.id, msg1_info.id);
    assert_eq!(found_info1.text_message_id, message1.id);

    // Act - Find info for recipient for message2
    let result2 = service.find_info_by_user_id_and_message_id(recipient_user.id, message2.id).await;

    // Assert for message2
    assert!(result2.is_ok(), "Failed to find info for message2: {:?}", result2.err());
    let found_info2 = result2.unwrap();
    assert_eq!(found_info2.id, msg2_info.id);
    assert_eq!(found_info2.text_message_id, message2.id);

    // Verify they are different infos
    assert_ne!(found_info1.id, found_info2.id);

    // Cleanup
    common::cleanup_text_message_info(msg1_info.id).await;
    common::cleanup_text_message_info(msg2_info.id).await;
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(recipient_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}
