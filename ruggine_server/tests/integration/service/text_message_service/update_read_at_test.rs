use ruggine_server::service::text_message_service::TextMessageServiceTrait;
use ruggine_server::factory::text_message_factory::TextMessageFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::text_message_error::TextMessageError;
use chrono::{DateTime, SubsecRound, Utc};
use crate::common;
use ruggine_server::utils::service_initializer::ServiceInitializer;

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_success() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (recipient_user, _) = common::create_test_user("update_read_at_recipient").await;
    let (sender_user, _) = common::create_test_user("update_read_at_sender").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_read_at_group",
        sender_user.id,
    ).await;

    // Add recipient to the group
    let _recipient_membership = common::add_test_user_to_a_group(recipient_user.id, &group_chat).await;

    // Create a test message from sender
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for update read at".to_string())
    ).await;

    // Create text message info for recipient
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        recipient_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // Verify initial state (read_at should be None)
    assert!(created_info.read_at.is_none(), "Initial read_at should be None");

    // Prepare update payload
    let new_read_at = Utc::now();
    let update_read_at_payload = TextMessageFactory::fake_text_message_read_at_dto_update_with_ids(
        text_message.id,
        new_read_at
    );
    let update_sent_at_payload = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
        new_read_at - chrono::Duration::minutes(15)
    );
    service.update_sent_at(recipient_user.id, update_sent_at_payload.clone()).await.unwrap();
    // Act
    let result = service.update_read_at(recipient_user.id, update_read_at_payload).await;

    // Assert
    assert!(result.is_ok(), "Failed to update read_at: {:?}", result.err());
    let updated_info = result.unwrap();
    
    assert_eq!(updated_info.id, created_info.id);
    assert_eq!(updated_info.user_id, recipient_user.id);
    assert_eq!(updated_info.text_message_id, text_message.id);
    assert!(updated_info.read_at.is_some(), "read_at should be set");
    assert_eq!(
        updated_info.sent_at.unwrap().trunc_subsecs(6),
        update_sent_at_payload.sent_at.trunc_subsecs(6)
    ); // Should remain unchanged

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(recipient_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_message_not_found() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (user, _) = common::create_test_user("update_read_at_not_found").await;
    
    let non_existing_message_id = 999999;
    let update_payload = TextMessageFactory::fake_text_message_read_at_dto_update_with_ids(
        non_existing_message_id,
        Utc::now()
    );

    // Act
    let result = service.update_read_at(user.id, update_payload).await;

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
async fn test_update_read_at_unauthorized_user() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("update_read_at_sender_unauth").await;
    let (recipient_user, _) = common::create_test_user("update_read_at_recipient_unauth").await;
    let (unauthorized_user, _) = common::create_test_user("update_read_at_unauthorized").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_read_at_group_unauth",
        sender_user.id,
    ).await;

    // Add recipient to the group
    let _recipient_membership = common::add_test_user_to_a_group(recipient_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for unauthorized read update".to_string())
    ).await;

    // Create text message info for recipient
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        recipient_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // Prepare update payload
    let update_payload = TextMessageFactory::fake_text_message_read_at_dto_update_with_ids(
        text_message.id,
        Utc::now()
    );

    // Act - try to update with unauthorized user
    let result = service.update_read_at(unauthorized_user.id, update_payload).await;

    // Assert
    assert!(result.is_err(), "Expected error for unauthorized user");

    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
            // Expected error type - user doesn't have message info for this message
        }
        e => panic!("Expected UserCannotAccessMessages error, gor {:?}", e),
    }

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(recipient_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
    common::cleanup_user(unauthorized_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_already_set() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (recipient_user, _) = common::create_test_user("update_read_at_already_set").await;
    let (sender_user, _) = common::create_test_user("update_read_at_sender_already_set").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_read_at_group_already_set",
        sender_user.id,
    ).await;

    // Add recipient to the group
    let _recipient_membership = common::add_test_user_to_a_group(recipient_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for already set read_at".to_string())
    ).await;

    // Create text message info
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        recipient_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // First update
    let first_read_at = Utc::now();
    let first_read_at_update_payload = TextMessageFactory::fake_text_message_read_at_dto_update_with_ids(
        text_message.id,
        first_read_at
    );
    let first_sent_at_update_payload = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
        first_read_at
    );
    service.update_sent_at(recipient_user.id, first_sent_at_update_payload).await.unwrap();
    let first_result = service.update_read_at(recipient_user.id, first_read_at_update_payload).await;
    assert!(first_result.is_ok(), "First update should succeed");

    // Second update with different timestamp
    let second_read_at = Utc::now();
    let second_read_at_update_payload = TextMessageFactory::fake_text_message_read_at_dto_update_with_ids(
        text_message.id,
        second_read_at
    );
    // Act
    let result = service.update_read_at(recipient_user.id, second_read_at_update_payload).await;

    // Assert
    assert!(result.is_err(), "Second update should not succeed");

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(recipient_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_preserves_sent_at() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (recipient_user, _) = common::create_test_user("update_read_at_preserve_sent").await;
    let (sender_user, _) = common::create_test_user("update_read_at_sender_preserve").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_read_at_group_preserve",
        sender_user.id,
    ).await;

    // Add recipient to the group
    let _recipient_membership = common::add_test_user_to_a_group(recipient_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for preserving sent_at".to_string())
    ).await;

    // Create text message info
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        recipient_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // First, update sent_at
    let sent_at = Utc::now();
    let sent_update_payload = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
        sent_at
    );
    let sent_result = service.update_sent_at(recipient_user.id, sent_update_payload).await;
    assert!(sent_result.is_ok(), "Sent at update should succeed");

    // Now update read_at
    let read_at = Utc::now();
    let read_update_payload = TextMessageFactory::fake_text_message_read_at_dto_update_with_ids(
        text_message.id,
        read_at
    );

    // Act
    let result = service.update_read_at(recipient_user.id, read_update_payload).await;

    // Assert
    assert!(result.is_ok(), "Read at update should succeed");
    let updated_info = result.unwrap();
    
    assert_eq!(updated_info.id, created_info.id);
    assert_eq!(updated_info.user_id, recipient_user.id);
    assert_eq!(updated_info.text_message_id, text_message.id);
    assert!(updated_info.read_at.is_some(), "read_at should be set");
    assert!(updated_info.sent_at.is_some(), "sent_at should be preserved");

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(recipient_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_multiple_users_same_message() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("update_read_at_multi_sender").await;
    let (recipient1_user, _) = common::create_test_user("update_read_at_multi_recipient1").await;
    let (recipient2_user, _) = common::create_test_user("update_read_at_multi_recipient2").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_read_at_group_multi",
        sender_user.id,
    ).await;

    // Add recipients to the group
    let _recipient1_membership = common::add_test_user_to_a_group(recipient1_user.id, &group_chat).await;
    let _recipient2_membership = common::add_test_user_to_a_group(recipient2_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for multiple recipients read update".to_string())
    ).await;

    // Create text message info for both recipients
    let create_payload1 = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        recipient1_user.id, 
        text_message.id
    );
    let created_info1 = service.create_info(create_payload1, sender_user.id).await.unwrap();

    let create_payload2 = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        recipient2_user.id, 
        text_message.id
    );
    let created_info2 = service.create_info(create_payload2, sender_user.id).await.unwrap();

    // Update read_at for first recipient
    let read_at1 = Utc::now();
    let update_sent_at_payload1 = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
        read_at1
    );
    let update_read_at_payload1 = TextMessageFactory::fake_text_message_read_at_dto_update_with_ids(
        text_message.id,
        read_at1
    );
    service.update_sent_at(recipient1_user.id, update_sent_at_payload1).await.unwrap();
    let result1 = service.update_read_at(recipient1_user.id, update_read_at_payload1).await;
    assert!(result1.is_ok(), "First recipient read update should succeed");

    // Update read_at for second recipient  
    let read_at2 = Utc::now();
    let update_read_at_payload2 = TextMessageFactory::fake_text_message_read_at_dto_update_with_ids(
        text_message.id,
        read_at2
    );
    let update_sent_at_payload2 = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
        read_at2
    );
    service.update_sent_at(recipient2_user.id, update_sent_at_payload2).await.unwrap();
    // Act
    let result2 = service.update_read_at(recipient2_user.id, update_read_at_payload2).await;

    // Assert
    assert!(result2.is_ok(), "Second recipient read update should succeed");
    let updated_info2 = result2.unwrap();
    
    // Verify that the second update only affected the second recipient's info
    assert_eq!(updated_info2.id, created_info2.id);
    assert_eq!(updated_info2.user_id, recipient2_user.id);
    assert_eq!(updated_info2.text_message_id, text_message.id);
    assert!(updated_info2.read_at.is_some(), "read_at should be set for second recipient");

    // Verify first recipient's info is still accessible and unchanged by second update
    let first_info_after = service.find_info_by_id(created_info1.id).await.unwrap();
    assert_eq!(first_info_after.user_id, recipient1_user.id);
    assert!(first_info_after.read_at.is_some(), "First recipient's read_at should still be set");

    // Cleanup
    common::cleanup_text_message_info(created_info1.id).await;
    common::cleanup_text_message_info(created_info2.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(recipient1_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(recipient2_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient1_user.id).await;
    common::cleanup_user(recipient2_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_cannot_update_again() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (recipient_user, _) = common::create_test_user("update_read_at_cannot_update_again").await;
    let (sender_user, _) = common::create_test_user("update_read_at_sender_cannot_update_again").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_read_at_group_cannot_update_again",
        sender_user.id,
    ).await;

    // Add recipient to the group
    let _recipient_membership = common::add_test_user_to_a_group(recipient_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for cannot update read_at again".to_string())
    ).await;

    // Create text message info
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        recipient_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // First, set sent_at
    let sent_at = Utc::now();
    let sent_update_payload = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
        sent_at
    );
    let sent_result = service.update_sent_at(recipient_user.id, sent_update_payload).await;
    assert!(sent_result.is_ok(), "Sent at update should succeed");

    // First read_at update
    let read_at = Utc::now();
    let read_update_payload = TextMessageFactory::fake_text_message_read_at_dto_update_with_ids(
        text_message.id,
        read_at
    );
    let first_result = service.update_read_at(recipient_user.id, read_update_payload).await;
    assert!(first_result.is_ok(), "First read at update should succeed");

    // Second read_at update attempt
    let second_read_at = Utc::now();
    let second_read_update_payload = TextMessageFactory::fake_text_message_read_at_dto_update_with_ids(
        text_message.id,
        second_read_at
    );

    // Act
    let result = service.update_read_at(recipient_user.id, second_read_update_payload).await;

    // Assert
    assert!(result.is_err(), "Second read_at update should fail");

    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::CannotUpdateReadAtAgain) => {
            // Expected error type
        }
        e => panic!("Expected CannotUpdateReadAtAgain error, got {:?}", e),
    }

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(recipient_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_cannot_set_before_sent_at() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (recipient_user, _) = common::create_test_user("update_read_at_cannot_set_before_sent").await;
    let (sender_user, _) = common::create_test_user("update_read_at_sender_cannot_set_before_sent").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_read_at_group_cannot_set_before_sent",
        sender_user.id,
    ).await;

    // Add recipient to the group
    let _recipient_membership = common::add_test_user_to_a_group(recipient_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for cannot set read_at before sent_at".to_string())
    ).await;

    // Create text message info
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        recipient_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // Verify initial state (sent_at should be None)
    assert!(created_info.sent_at.is_none(), "Initial sent_at should be None");

    // Attempt to update read_at without setting sent_at first
    let read_at = Utc::now();
    let read_update_payload = TextMessageFactory::fake_text_message_read_at_dto_update_with_ids(
        text_message.id,
        read_at
    );

    // Act
    let result = service.update_read_at(recipient_user.id, read_update_payload).await;

    // Assert
    assert!(result.is_err(), "Read_at update should fail when sent_at is None");

    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::CannotSetReadAtBeforeSentAt) => {
            // Expected error type
        }
        e => panic!("Expected CannotSetReadAtBeforeSentAt error, got {:?}", e),
    }

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(recipient_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_must_be_greater_or_equal_to_sent_at() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (recipient_user, _) = common::create_test_user("update_read_at_must_be_greater").await;
    let (sender_user, _) = common::create_test_user("update_read_at_sender_must_be_greater").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_read_at_group_must_be_greater",
        sender_user.id,
    ).await;

    // Add recipient to the group
    let _recipient_membership = common::add_test_user_to_a_group(recipient_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for read_at must be greater than sent_at".to_string())
    ).await;

    // Create text message info
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        recipient_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // Set sent_at to current time
    let sent_at = Utc::now();
    let sent_update_payload = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
        sent_at
    );
    let sent_result = service.update_sent_at(recipient_user.id, sent_update_payload).await;
    assert!(sent_result.is_ok(), "Sent at update should succeed");

    // Attempt to update read_at with a time BEFORE sent_at
    let read_at_before_sent = sent_at - chrono::Duration::minutes(10);
    let read_update_payload = TextMessageFactory::fake_text_message_read_at_dto_update_with_ids(
        text_message.id,
        read_at_before_sent
    );

    // Act
    let result = service.update_read_at(recipient_user.id, read_update_payload).await;

    // Assert
    assert!(result.is_err(), "Read_at update should fail when read_at is before sent_at");

    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::ReadAtMustBeGreaterOrEqualsToSentAt) => {
            // Expected error type
        }
        e => panic!("Expected ReadAtMustBeGreaterOrEqualsToSentAt error, got {:?}", e),
    }

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(recipient_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_update_read_at_exactly_equal_to_sent_at() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (recipient_user, _) = common::create_test_user("update_read_at_equal_to_sent").await;
    let (sender_user, _) = common::create_test_user("update_read_at_sender_equal_to_sent").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "update_read_at_group_equal_to_sent",
        sender_user.id,
    ).await;

    // Add recipient to the group
    let _recipient_membership = common::add_test_user_to_a_group(recipient_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for read_at equal to sent_at".to_string())
    ).await;

    // Create text message info
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        recipient_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // Set sent_at to a specific time
    let timestamp = Utc::now().trunc_subsecs(6); // Truncate to avoid microsecond precision issues
    let sent_update_payload = TextMessageFactory::fake_text_message_sent_at_dto_update_with_ids(
        text_message.id,
        timestamp
    );
    let sent_result = service.update_sent_at(recipient_user.id, sent_update_payload).await;
    assert!(sent_result.is_ok(), "Sent at update should succeed");

    // Update read_at with the SAME time as sent_at (should be allowed)
    let read_update_payload = TextMessageFactory::fake_text_message_read_at_dto_update_with_ids(
        text_message.id,
        timestamp
    );

    // Act
    let result = service.update_read_at(recipient_user.id, read_update_payload).await;

    // Assert
    assert!(result.is_ok(), "Read_at update should succeed when read_at equals sent_at: {:?}", result.err());
    let updated_info = result.unwrap();
    
    assert_eq!(updated_info.id, created_info.id);
    assert_eq!(updated_info.user_id, recipient_user.id);
    assert_eq!(updated_info.text_message_id, text_message.id);
    assert!(updated_info.read_at.is_some(), "read_at should be set");
    assert!(updated_info.sent_at.is_some(), "sent_at should be preserved");
    assert_eq!(
        updated_info.read_at.unwrap().trunc_subsecs(6),
        updated_info.sent_at.unwrap().trunc_subsecs(6),
        "read_at should equal sent_at"
    );

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(recipient_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}
