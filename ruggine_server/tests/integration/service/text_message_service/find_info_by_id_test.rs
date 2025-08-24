use ruggine_server::service::text_message_service::TextMessageServiceTrait;
use ruggine_server::factory::text_message_factory::TextMessageFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::text_message_error::TextMessageError;
use crate::common;
use ruggine_server::utils::service_initializer::ServiceInitializer;

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_id_success() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_info_by_id_sender").await;
    
    // Create a test group with invitation and membership for the sender
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "find_info_by_id_group",
        sender_user.id,
    ).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for find info by id".to_string())
    ).await;

    // Create text message info using the service
    let create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let created_info = service.create_info(create_payload, sender_user.id).await.unwrap();

    // Act
    let result = service.find_info_by_id(created_info.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find text message info: {:?}", result.err());
    let found_info = result.unwrap();
    
    assert_eq!(found_info.id, created_info.id);
    assert_eq!(found_info.user_id, sender_user.id);
    assert_eq!(found_info.text_message_id, text_message.id);
    assert_eq!(found_info.sent_at, created_info.sent_at);
    assert_eq!(found_info.read_at, created_info.read_at);

    // Cleanup
    common::cleanup_text_message_info(created_info.id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_test_user_from_a_group_chat(sender_user.id, group_chat.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_id_not_found() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();
    
    let non_existing_id = 999999;

    // Act
    let result = service.find_info_by_id(non_existing_id).await;

    // Assert
    assert!(result.is_err(), "Expected error for non-existing message info");

    match result.unwrap_err() {
        ApiError::TextMessageError(TextMessageError::MessageInfoNotFound) => {
            // Expected error type
        }
        _ => panic!("Expected MessageInfoNotFound error"),
    }
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_id_multiple_infos_same_message() {
    // Arrange
    let db = common::get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("multi_info_sender").await;
    let (recipient_user, _) = common::create_test_user("multi_info_recipient").await;
    
    // Create a test group with invitation and membership
    let group_chat = common::create_test_group_chat_with_invitation_and_membership(
        "multi_info_group",
        sender_user.id,
    ).await;
    
    // Add recipient to the group
    let _recipient_membership = common::add_test_user_to_a_group(recipient_user.id, &group_chat).await;

    // Create a test message
    let text_message = common::create_test_text_message(
        sender_user.id, 
        group_chat.id, 
        Some("Test message for multiple infos".to_string())
    ).await;

    // Create info for sender
    let sender_create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        sender_user.id, 
        text_message.id
    );
    let sender_info = service.create_info(sender_create_payload, sender_user.id).await.unwrap();

    // Create info for recipient
    let recipient_create_payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(
        recipient_user.id, 
        text_message.id
    );
    let recipient_info = service.create_info(recipient_create_payload, sender_user.id).await.unwrap();

    // Act & Assert for sender info
    let sender_result = service.find_info_by_id(sender_info.id).await;
    assert!(sender_result.is_ok());
    let found_sender_info = sender_result.unwrap();
    assert_eq!(found_sender_info.user_id, sender_user.id);
    assert_eq!(found_sender_info.text_message_id, text_message.id);

    // Act & Assert for recipient info
    let recipient_result = service.find_info_by_id(recipient_info.id).await;
    assert!(recipient_result.is_ok());
    let found_recipient_info = recipient_result.unwrap();
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
