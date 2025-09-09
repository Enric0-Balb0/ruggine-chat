use ruggine_server::service::text_message_service::TextMessageServiceTrait;
use ruggine_server::factory::text_message_factory::TextMessageFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::text_message_error::TextMessageError;
use crate::common;
use ruggine_server::utils::service_initializer::ServiceInitializer;

#[tokio_shared_rt::test(shared)]
async fn test_update_sent_at_success() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("update_sent_at_sender").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_sent_at_group",
        sender_user.id,
    ).await;

    // Create a test message
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for update sent at".to_string())
    ).await;

    // Create text message info using the service
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // Verify initial state (sent_at should be None)
    assert!(created_info.sent_at.is_none(), "Initial sent_at should be None");

    // Prepare update payload
    let update_payload = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
    );

    // Act
    let result = service.update_sent_at(sender_user.id, update_payload).await;

    // Assert
    assert!(result.is_ok(), "Failed to update sent_at: {:?}", result.err());
    let updated_info = result.unwrap();
    
    assert_eq!(updated_info.id, created_info.id);
    assert_eq!(updated_info.user_id, sender_user.id);
    assert_eq!(updated_info.text_message_id, text_message.id);
    assert!(updated_info.sent_at.is_some(), "sent_at should be set");
    assert_eq!(updated_info.read_at, created_info.read_at); // read_at should remain unchanged

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_update_sent_at_message_not_found() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (user, _) = common::create_test_user("update_sent_at_not_found").await;
    
    let non_existing_message_id = 999999;
    let update_payload = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        non_existing_message_id,
    );

    // Act
    let result = service.update_sent_at(user.id, update_payload).await;

    // Assert
    assert!(result.is_err(), "Expected error for non-existing message");

    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::MessageNotFound) => {
            // Expected error type
        }
        e => panic!("Expected MessageNotFound error, got {:?}", e),
    }

    // Cleanup
    common::cleanup_user(user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_update_sent_at_unauthorized_user() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("update_sent_at_sender_unauth").await;
    let (unauthorized_user, _) = common::create_test_user("update_sent_at_unauthorized").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_sent_at_group_unauth",
        sender_user.id,
    ).await;

    // Create a test message
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for unauthorized update".to_string())
    ).await;

    // Create text message info for sender
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // Prepare update payload
    let update_payload = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
    );

    // Act - try to update with unauthorized user
    let result = service.update_sent_at(unauthorized_user.id, update_payload).await;

    // Assert
    assert!(result.is_err(), "Expected error for unauthorized user");

    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
            // Expected error type - user doesn't have message info for this message
        }
        e => panic!("Expected UserCannotAccessMessages error, got {:?}", e),
    }

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(unauthorized_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_update_sent_at_already_set() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("update_sent_at_already_set").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_sent_at_group_already_set",
        sender_user.id,
    ).await;

    // Create a test message
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for already set sent_at".to_string())
    ).await;

    // Create text message info
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // First update
    let first_update_payload = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
    );
    let first_result = service.update_sent_at(sender_user.id, first_update_payload).await;
    assert!(first_result.is_ok(), "First update should succeed");

    // Second update with different timestamp
    let second_update_payload = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
    );

    // Act
    let result = service.update_sent_at(sender_user.id, second_update_payload).await;

    // Assert
    assert!(result.is_err(), "Second update should not succeed");

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_update_sent_at_preserves_read_at() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("update_sent_at_preserve_read").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_sent_at_group_preserve",
        sender_user.id,
    ).await;

    // Create a test message
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for preserving read_at".to_string())
    ).await;

    // Create text message info
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // Now update sent_at
    let sent_update_payload = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
    );

    // Act
    let result = service.update_sent_at(sender_user.id, sent_update_payload).await;

    // Assert
    assert!(result.is_ok(), "Sent at update should succeed, but got: {:?}", result.unwrap_err());
    let updated_info = result.unwrap();
    
    assert_eq!(updated_info.id, created_info.id);
    assert_eq!(updated_info.user_id, sender_user.id);
    assert_eq!(updated_info.text_message_id, text_message.id);
    assert!(updated_info.sent_at.is_some(), "sent_at should be set");
    assert!(updated_info.read_at.is_none(), "read_at should be preserved");

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

/*
#[tokio_shared_rt::test(shared)]
async fn test_update_sent_at_must_be_less_or_equal_to_now() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("update_sent_at_future").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_sent_at_group_future",
        sender_user.id,
    ).await;

    // Create a test message
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for future sent_at".to_string())
    ).await;

    // Create text message info
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // Attempt to set sent_at to a future time
    let update_payload = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
    );

    // Act
    let result = service.update_sent_at(sender_user.id, update_payload).await;

    // Assert
    assert!(result.is_err(), "Sent_at update should fail when sent_at is in the future");

    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::SentAtMustBeLessOrEqualsToNowAndGreaterThanMessageCreation) => {
            // Expected error type
        }
        e => panic!("Expected SentAtMustBeLessOrEqualsToNow error, got {:?}", e),
    }

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}
 */
