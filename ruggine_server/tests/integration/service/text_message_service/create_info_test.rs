use ruggine_server::service::text_message_service::TextMessageServiceTrait;
use ruggine_server::factory::text_message_factory::TextMessageFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::text_message_error::TextMessageError;
use crate::common;
use ruggine_server::utils::service_initializer::ServiceInitializer;

#[tokio_shared_rt::test(shared)]
async fn test_create_info_success() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("create_info_sender").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "create_info_success_group",
        sender_user.id,
    ).await;

    // Create a test message
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for info creation".to_string())
    ).await;

    let payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );

    // Act
    let result = service.create_info(payload.clone(), sender_user.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to create text message info: {:?}", result.err());
    let created_info = result.unwrap();
    
    assert_eq!(created_info.user_id, sender_user.id);
    assert_eq!(created_info.text_message_id, text_message.id);
    assert!(created_info.id > 0);
    assert!(created_info.sent_at.is_none());
    assert!(created_info.read_at.is_none()); // Should be None initially

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_create_info_message_not_found() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("create_info_no_message").await;
    let nonexistent_message_id = 99999;
    
    let payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        nonexistent_message_id
    );

    // Act
    let result = service.create_info(payload, sender_user.id).await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, ApiError::TextMessageError(TextMessageError::MessageNotFound)));

    // Cleanup
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_create_info_user_not_creator() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (message_creator, _) = common::create_test_user("create_info_creator").await;
    let (other_user, _) = common::create_test_user("create_info_other").await;
    
    // Create a test group with invitation and membership for the creator
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "create_info_creator_group",
        message_creator.id,
    ).await;

    // Create a test message by the creator
    let text_message = common::create_test_text_message_without_message_info(
        message_creator.id, 
        group_chat.id, 
        Some("Test message by creator".to_string())
    ).await;

    // Try to create info with a different user (not the creator)
    let payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        other_user.id, 
        text_message.id
    );

    // Act
    let result = service.create_info(payload, other_user.id).await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, ApiError::TextMessageError(TextMessageError::UserIsNotCreator)));

    // Cleanup
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(message_creator.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(message_creator.id).await;
    common::cleanup_user(other_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_create_info_multiple_infos_same_message() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("create_info_multi_sender").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "create_info_multi_group",
        sender_user.id,
    ).await;

    // Create a test message
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for multiple infos".to_string())
    ).await;

    let payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );

    // Act - Create first info
    let result1 = service.create_info(payload.clone(), sender_user.id).await;
    
    // Act - Try to create second info for the same message and user
    let result2 = service.create_info(payload, sender_user.id).await;

    // Assert - First should succeed
    assert!(result1.is_ok(), "Failed to create first text message info: {:?}", result1.err());
    let created_info1 = result1.unwrap();
    
    // Assert - Second should also succeed (multiple infos for same user and message not allowed)
    assert!(result2.is_err(), "Failed to create second text message info: {:?}", result2.err());

    // Cleanup
    common::cleanup_text_message_info(created_info1.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_create_info_with_complex_scenario() {
    // Arrange - Create a more complex scenario with multiple users and groups
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (group_owner, _) = common::create_test_user("create_info_complex_owner").await;
    let (message_sender, _) = common::create_test_user("create_info_complex_sender").await;
    
    // Create a test group with owner
    let group_chat = common::create_test_group_chat("create_info_complex_group", group_owner.id).await;
    
    // Add sender as member to the group
    let invitation = common::create_test_invitation(
        group_owner.id,
        message_sender.id,
        group_chat.id,
    ).await;
    common::accept_test_invitation(invitation.id).await;
    let _membership = common::create_test_group_membership(invitation.id, message_sender.id).await;

    // Create multiple messages
    let message1 = common::create_test_text_message_without_message_info(
        message_sender.id, 
        group_chat.id, 
        Some("First message".to_string())
    ).await;
    
    let message2 = common::create_test_text_message_without_message_info(
        message_sender.id, 
        group_chat.id, 
        Some("Second message".to_string())
    ).await;

    // Create payloads for both messages
    let payload1 = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        message_sender.id, 
        message1.id
    );
    let payload2 = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        message_sender.id, 
        message2.id
    );

    // Act - Create infos for both messages
    let result1 = service.create_info(payload1, message_sender.id).await;
    let result2 = service.create_info(payload2, message_sender.id).await;

    // Assert
    assert!(result1.is_ok(), "Failed to create info for first message: {:?}", result1.err());
    assert!(result2.is_ok(), "Failed to create info for second message: {:?}", result2.err());
    
    let info1 = result1.unwrap();
    let info2 = result2.unwrap();
    
    assert_eq!(info1.text_message_id, message1.id);
    assert_eq!(info2.text_message_id, message2.id);
    assert_eq!(info1.user_id, message_sender.id);
    assert_eq!(info2.user_id, message_sender.id);

    // Cleanup
    common::cleanup_text_message_info(info1.id).await;
    common::cleanup_text_message_info(info2.id).await;
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_test_user_from_a_group_chat(message_sender.id, group_chat.id).await;
    common::cleanup_test_user_from_a_group_chat(group_owner.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(message_sender.id).await;
    common::cleanup_user(group_owner.id).await;
}
