use crate::common;
use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};
use ruggine_server::factory::text_message_factory::TextMessageFactory;

#[tokio_shared_rt::test(shared)]
async fn test_insert_success() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group first
    let (sender_user, _) = common::create_test_user("text_msg_sender").await;
    let group_chat = common::create_test_group_chat("text_msg_group", sender_user.id).await;
    
    let new_message = TextMessageFactory::unique_fake_new_text_message(
        "integration_test",
        sender_user.id,
        group_chat.id
    );

    // Act
    let result = repository.insert(new_message.clone()).await;

    // Assert
    assert!(result.is_ok(), "Failed to insert text message: {:?}", result.err());
    let message_id = result.unwrap();
    assert!(message_id > 0, "Message ID should be positive");

    // Verify the message was actually inserted
    let inserted_message = repository.find(message_id).await;
    assert!(inserted_message.is_ok(), "Failed to find inserted message");
    
    let message = inserted_message.unwrap();
    assert_eq!(message.content, new_message.content);
    assert_eq!(message.sender_id, new_message.sender_id);
    assert_eq!(message.group_chat_id, new_message.group_chat_id);

    // Cleanup
    common::cleanup_text_message(message_id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_insert_foreign_key_violation() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    let new_message = TextMessageFactory::fake_new_text_message_with_ids(999999, 999999); // Non-existent IDs

    // Act
    let result = repository.insert(new_message).await;

    // Assert
    assert!(result.is_err(), "Expected foreign key constraint violation");
}

#[tokio_shared_rt::test(shared)]
async fn test_insert_with_long_content() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    let (sender_user, _) = common::create_test_user("long_content_sender").await;
    let group_chat = common::create_test_group_chat("long_content_group", sender_user.id).await;
    
    let long_content = "a".repeat(5000); // Long but reasonable content
    let new_message = TextMessageFactory::fake_new_text_message_with_content(long_content.clone());
    let new_message = TextMessageFactory::with_sender_id(new_message, sender_user.id);
    let new_message = TextMessageFactory::with_group_chat_id(new_message, group_chat.id);

    // Act
    let result = repository.insert(new_message).await;

    // Assert
    assert!(result.is_ok(), "Failed to insert message with long content");
    let message_id = result.unwrap();
    
    let inserted_message = repository.find(message_id).await.unwrap();
    assert_eq!(inserted_message.content, long_content);

    // Cleanup
    common::cleanup_text_message(message_id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}
