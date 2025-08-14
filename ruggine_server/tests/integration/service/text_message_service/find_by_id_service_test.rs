use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};
use ruggine_server::service::text_message_service::{TextMessageService, TextMessageServiceTrait};
use crate::common;
use std::sync::Arc;
use ruggine_server::utils::service_initializer::ServiceInitializer;

#[tokio_shared_rt::test(shared)]
async fn test_find_by_id_existing_message() {
    // Arrange
    let db = common::get_database().await;
    let service_init  = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("find_by_id_sender").await;
    let group_chat = common::create_test_group_chat("find_by_id_group", sender_user.id).await;
    
    // Create a message in the database
    let test_message = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Integration test message for find_by_id".to_string())
    ).await;

    // Act
    let result = service.find_by_id(test_message.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find text message: {:?}", result.err());
    let found_message_dto = result.unwrap();
    
    assert_eq!(found_message_dto.id, test_message.id);
    assert_eq!(found_message_dto.content, test_message.content);
    assert_eq!(found_message_dto.sender_id, test_message.sender_id);
    assert_eq!(found_message_dto.group_chat_id, test_message.group_chat_id);
    assert_eq!(found_message_dto.sent_at, test_message.sent_at);

    // Cleanup
    common::cleanup_text_message(test_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_id_non_existing_message() {
    // Arrange
    let db = common::get_database().await;
    let service_init  = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();
    
    let non_existing_id = 999999;

    // Act
    let result = service.find_by_id(non_existing_id).await;

    // Assert
    assert!(result.is_err(), "Expected error for non-existing message");
    
    match result.unwrap_err() {
        ruggine_server::error::api_error::ApiError::TextMessageError(
            ruggine_server::error::text_message_error::TextMessageError::MessageNotFound
        ) => {
            // Expected error type
        }
        _ => panic!("Expected MessageNotFound error"),
    }
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_id_multiple_messages_same_group() {
    // Arrange
    let db = common::get_database().await;
    let service_init  = ServiceInitializer::new(&db);
    let service = service_init.text_message_service();

    let (sender_user, _) = common::create_test_user("multi_msg_sender").await;
    let group_chat = common::create_test_group_chat("multi_msg_group", sender_user.id).await;
    
    // Create multiple messages
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

    // Act & Assert for message1
    let result1 = service.find_by_id(message1.id).await;
    assert!(result1.is_ok());
    let found_dto1 = result1.unwrap();
    assert_eq!(found_dto1.content, "First message");

    // Act & Assert for message2
    let result2 = service.find_by_id(message2.id).await;
    assert!(result2.is_ok());
    let found_dto2 = result2.unwrap();
    assert_eq!(found_dto2.content, "Second message");

    // Cleanup
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}
