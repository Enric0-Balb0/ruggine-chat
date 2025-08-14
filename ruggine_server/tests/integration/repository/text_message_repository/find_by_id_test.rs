use crate::common;
use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};
use ruggine_server::factory::text_message_factory::TextMessageFactory;

#[tokio_shared_rt::test(shared)]
async fn test_find_success() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    let (sender_user, _) = common::create_test_user("find_sender").await;
    let group_chat = common::create_test_group_chat("find_group", sender_user.id).await;
    
    // Create a message in the database
    let test_message = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Integration test message".to_string())
    ).await;

    // Act
    let result = repository.find(test_message.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find text message: {:?}", result.err());
    let found_message = result.unwrap();
    
    assert_eq!(found_message.id, test_message.id);
    assert_eq!(found_message.content, test_message.content);
    assert_eq!(found_message.sender_id, test_message.sender_id);
    assert_eq!(found_message.group_chat_id, test_message.group_chat_id);

    // Cleanup
    common::cleanup_text_message(test_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user_by_id(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_not_found() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    let non_existent_id = 999999;

    // Act
    let result = repository.find(non_existent_id).await;

    // Assert
    assert!(result.is_err(), "Expected not found error");
    matches!(result.unwrap_err(), sqlx::Error::RowNotFound);
}