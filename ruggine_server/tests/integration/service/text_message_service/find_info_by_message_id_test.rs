use ruggine_server::service::text_message_service::TextMessageServiceTrait;
use ruggine_server::factory::text_message_factory::TextMessageFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::text_message_error::TextMessageError;
use ruggine_server::error::group_membership_error::GroupMembershipError;
use crate::common;
use ruggine_server::utils::service_initializer::ServiceInitializer;

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_message_id_success() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_info_by_msg_sender").await;
    let (recipient_user, _) = common::create_test_user("find_info_by_msg_recipient").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_info_by_msg_group",
        sender_user.id,
    ).await;

    // Add recipient to the group
    let _recipient_membership = common::add_test_user_to_a_group(recipient_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for find info by message id".to_string())
    ).await;

    // Create text message info for sender
    let sender_create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let sender_info = service.create_info(sender_create_payload, sender_user.id).await.unwrap();

    // Create text message info for recipient
    let recipient_create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        recipient_user.id, 
        text_message.id
    );
    let recipient_info = service.create_info(recipient_create_payload, sender_user.id).await.unwrap();

    // Act
    let result = service.find_info_by_message_id(sender_user.id, text_message.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find text message infos: {:?}", result.err());
    let found_infos = result.unwrap();
    
    assert_eq!(found_infos.len(), 2);
    
    // Verify both infos are returned (order may vary)
    let info_ids: Vec<i32> = found_infos.iter().map(|info| info.id).collect();
    assert!(info_ids.contains(&sender_info.id));
    assert!(info_ids.contains(&recipient_info.id));

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
async fn test_find_info_by_message_id_empty_result() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_info_empty_sender").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_info_empty_group",
        sender_user.id,
    ).await;

    // Create a test message but no message info
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message with no info".to_string())
    ).await;

    // Act
    let result = service.find_info_by_message_id(sender_user.id, text_message.id).await;

    // Assert
    assert!(result.is_ok(), "Should succeed even with empty result: {:?}", result.err());
    let found_infos = result.unwrap();
    
    assert_eq!(found_infos.len(), 0);

    // Cleanup
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_message_id_message_not_found() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (user, _) = common::create_test_user("find_info_not_found_user").await;
    
    let non_existing_message_id = 999999;

    // Act
    let result = service.find_info_by_message_id(non_existing_message_id, user.id).await;

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
async fn test_find_info_by_message_id_access_denied_not_member() {
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
    let result = service.find_info_by_message_id(unauthorized_user.id, text_message.id).await;

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
async fn test_find_info_by_message_id_access_denied_left_member() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_info_left_sender").await;
    let (left_user, _) = common::create_test_user("find_info_left_user").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_info_left_group",
        sender_user.id,
    ).await;

    // Add left_user to the group then remove them
    let _left_membership = common::add_test_user_to_a_group(left_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for left member".to_string())
    ).await;

    // User leaves the group
    common::test_user_leave_from_a_group(left_user.id, group_chat.id).await;

    // Act - Try to access with left user
    let result = service.find_info_by_message_id(left_user.id, text_message.id).await;

    // Assert
    assert!(result.is_err(), "Expected access denied error for left member");

    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
            // Expected error type
        }
        _ => panic!("Expected UserCannotAccessMessages error"),
    }

    // Cleanup
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(left_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(left_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_message_id_multiple_messages() {
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

    // Create text message info for both messages
    let msg1_info1 = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(sender_user.id, message1.id),
        sender_user.id
    ).await.unwrap();

    let msg1_info2 = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(recipient_user.id, message1.id),
        sender_user.id
    ).await.unwrap();

    let msg2_info = service.create_info(
        TextMessageFactory::fake_text_message_info_create_dto_with_ids(sender_user.id, message2.id),
        sender_user.id
    ).await.unwrap();

    // Act - Find info for message1 only
    let result1 = service.find_info_by_message_id(recipient_user.id, message1.id).await;

    // Assert
    assert!(result1.is_ok(), "Failed to find infos for message1: {:?}", result1.err());
    let found_infos1 = result1.unwrap();
    
    assert_eq!(found_infos1.len(), 2);
    let info_ids1: Vec<i32> = found_infos1.iter().map(|info| info.id).collect();
    assert!(info_ids1.contains(&msg1_info1.id));
    assert!(info_ids1.contains(&msg1_info2.id));
    assert!(!info_ids1.contains(&msg2_info.id)); // Should not contain info from different message

    // Act - Find info for message2 only
    let result2 = service.find_info_by_message_id(sender_user.id, message2.id).await;

    // Assert
    assert!(result2.is_ok(), "Failed to find infos for message2: {:?}", result2.err());
    let found_infos2 = result2.unwrap();
    
    assert_eq!(found_infos2.len(), 1);
    assert_eq!(found_infos2[0].id, msg2_info.id);

    // Cleanup
    common::cleanup_text_message_info(msg1_info1.id).await;
    common::cleanup_text_message_info(msg1_info2.id).await;
    common::cleanup_text_message_info(msg2_info.id).await;
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(recipient_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}
