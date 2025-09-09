use ruggine_server::entity::text_message::TextMessageInfoUpdate;
use crate::common;
use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};
use ruggine_server::factory::text_message_factory::TextMessageFactory;

#[tokio_shared_rt::test(shared)]
async fn test_find_first_message_with_no_sent_at_success() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("no_sent_sender").await;
    let (recipient_user, _) = common::create_test_user("no_sent_recipient").await;
    let group_chat = common::create_test_group_chat("no_sent_group", sender_user.id).await;

    // Create multiple text messages
    let first_message = common::create_test_text_message_without_message_info(
        sender_user.id,
        group_chat.id,
        Some("First message".to_string())
    ).await;

    let second_message = common::create_test_text_message_without_message_info(
        sender_user.id,
        group_chat.id,
        Some("Second message".to_string())
    ).await;

    // Create text message info with no sent_at for the recipient
    let first_message_info = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user.id,
        first_message.id
    );
    let first_info_id = repository.insert_text_message_info(first_message_info).await.unwrap();

    let second_message_info = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user.id,
        second_message.id
    );
    let second_info_id = repository.insert_text_message_info(second_message_info).await.unwrap();

    // Act
    let result = repository.find_first_message_with_no_sent_at(recipient_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find first message with no sent_at: {:?}", result.err());
    let found_message = result.unwrap();
    
    assert!(found_message.is_some(), "Expected to find a message with no sent_at");
    let message = found_message.unwrap();
    
    // Should return the oldest message (first one based on sent_at ordering)
    assert_eq!(message.id, first_message.id);
    assert_eq!(message.content, first_message.content);
    assert_eq!(message.sender_id, sender_user.id);
    assert_eq!(message.group_chat_id, group_chat.id);

    // Cleanup
    common::cleanup_text_message_info(first_info_id).await;
    common::cleanup_text_message_info(second_info_id).await;
    common::cleanup_text_message(first_message.id).await;
    common::cleanup_text_message(second_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_first_message_with_no_sent_at_all_messages_sent() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("all_sent_sender").await;
    let (recipient_user, _) = common::create_test_user("all_sent_recipient").await;
    let group_chat = common::create_test_group_chat("all_sent_group", sender_user.id).await;

    // Create a text message
    let message = common::create_test_text_message_without_message_info(
        sender_user.id,
        group_chat.id,
        Some("Test message".to_string())
    ).await;

    // Create text message info with sent_at already set
    let message_info = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user.id,
        message.id
    );
    let info_id = repository.insert_text_message_info(message_info).await.unwrap();

    // Update the info to have a sent_at timestamp
    repository.update_sent_at_info(info_id).await.unwrap();

    // Act
    let result = repository.find_first_message_with_no_sent_at(recipient_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to query for first message with no sent_at: {:?}", result.err());
    let found_message = result.unwrap();
    
    assert!(found_message.is_none(), "Expected no message with no sent_at when all messages are sent");

    // Cleanup
    common::cleanup_text_message_info(info_id).await;
    common::cleanup_text_message(message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_first_message_with_no_sent_at_no_messages() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    // Create test users and group chat without any messages
    let (sender_user, _) = common::create_test_user("no_msg_sender").await;
    let (recipient_user, _) = common::create_test_user("no_msg_recipient").await;
    let group_chat = common::create_test_group_chat("no_msg_group", sender_user.id).await;

    // Act
    let result = repository.find_first_message_with_no_sent_at(recipient_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to query for first message with no sent_at: {:?}", result.err());
    let found_message = result.unwrap();
    
    assert!(found_message.is_none(), "Expected no message when there are no messages in the group");

    // Cleanup
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_first_message_with_no_sent_at_different_group() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    // Create test users and group chats
    let (sender_user, _) = common::create_test_user("diff_group_sender").await;
    let (recipient_user, _) = common::create_test_user("diff_group_recipient").await;
    let group_chat_1 = common::create_test_group_chat("diff_group_1", sender_user.id).await;
    let group_chat_2 = common::create_test_group_chat("diff_group_2", sender_user.id).await;

    // Create a message in group_chat_1
    let message_in_group_1 = common::create_test_text_message_without_message_info(
        sender_user.id,
        group_chat_1.id,
        Some("Message in group 1".to_string())
    ).await;

    // Create text message info for recipient in group_chat_1
    let message_info = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user.id,
        message_in_group_1.id
    );
    let info_id = repository.insert_text_message_info(message_info).await.unwrap();

    // Act - search in group_chat_2 (should find nothing)
    let result = repository.find_first_message_with_no_sent_at(recipient_user.id, group_chat_2.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to query for first message with no sent_at: {:?}", result.err());
    let found_message = result.unwrap();
    
    assert!(found_message.is_none(), "Expected no message when searching in different group");

    // Cleanup
    common::cleanup_text_message_info(info_id).await;
    common::cleanup_text_message(message_in_group_1.id).await;
    common::cleanup_group_chat(group_chat_1.id).await;
    common::cleanup_group_chat(group_chat_2.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_first_message_with_no_sent_at_different_user() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("diff_user_sender").await;
    let (recipient_user_1, _) = common::create_test_user("diff_user_recipient_1").await;
    let (recipient_user_2, _) = common::create_test_user("diff_user_recipient_2").await;
    let group_chat = common::create_test_group_chat("diff_user_group", sender_user.id).await;

    // Create a message
    let message = common::create_test_text_message_without_message_info(
        sender_user.id,
        group_chat.id,
        Some("Test message".to_string())
    ).await;

    // Create text message info for recipient_user_1
    let message_info = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user_1.id,
        message.id
    );
    let info_id = repository.insert_text_message_info(message_info).await.unwrap();

    // Act - search for recipient_user_2 (should find nothing)
    let result = repository.find_first_message_with_no_sent_at(recipient_user_2.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to query for first message with no sent_at: {:?}", result.err());
    let found_message = result.unwrap();
    
    assert!(found_message.is_none(), "Expected no message when searching for different user");

    // Cleanup
    common::cleanup_text_message_info(info_id).await;
    common::cleanup_text_message(message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user_1.id).await;
    common::cleanup_user(recipient_user_2.id).await;
}
