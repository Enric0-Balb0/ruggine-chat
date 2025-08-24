use crate::common;
use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};
use ruggine_server::factory::text_message_factory::TextMessageFactory;

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_user_id_and_message_id_success() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("info_find_sender").await;
    let (recipient_user, _) = common::create_test_user("info_find_recipient").await;
    let group_chat = common::create_test_group_chat("info_find_group", sender_user.id).await;

    // Create a text message first
    let text_message = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Test message for find info".to_string())
    ).await;

    // Create text message info
    let text_message_info = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user.id,
        text_message.id
    );

    let info_id = repository.insert_text_message_info(text_message_info.clone()).await.unwrap();

    // Act
    let result = repository.find_info_by_user_id_and_message_id(recipient_user.id, text_message.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find text message info: {:?}", result.err());
    let found_info = result.unwrap();

    assert_eq!(found_info.id, info_id);
    assert_eq!(found_info.user_id, recipient_user.id);
    assert_eq!(found_info.text_message_id, text_message.id);

    // Cleanup
    common::cleanup_text_message_info(info_id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_user_id_and_message_id_not_found() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    let non_existent_id = 999999;

    // Act
    let result = repository.find_info_by_user_id_and_message_id(non_existent_id, non_existent_id).await;

    // Assert
    assert!(result.is_err(), "Expected not found error");
    matches!(result.unwrap_err(), sqlx::Error::RowNotFound);
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_user_id_and_message_id_multiple_infos_same_message() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("multi_info_sender").await;
    let (recipient1, _) = common::create_test_user("multi_info_recipient1").await;
    let (recipient2, _) = common::create_test_user("multi_info_recipient2").await;
    let group_chat = common::create_test_group_chat("multi_info_group", sender_user.id).await;

    // Create a text message
    let text_message = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Message for multiple recipients".to_string())
    ).await;

    // Create multiple message infos for different users but same message
    let info1 = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient1.id,
        text_message.id
    );
    let info2 = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient2.id,
        text_message.id
    );

    let info1_id = repository.insert_text_message_info(info1).await.unwrap();
    let info2_id = repository.insert_text_message_info(info2).await.unwrap();

    // Act - Find both infos individually
    let result1 = repository.find_info_by_user_id_and_message_id(recipient1.id, text_message.id).await;
    let result2 = repository.find_info_by_user_id_and_message_id(recipient2.id, text_message.id).await;

    // Assert
    assert!(result1.is_ok(), "Failed to find first text message info");
    assert!(result2.is_ok(), "Failed to find second text message info");

    let found_info1 = result1.unwrap();
    let found_info2 = result2.unwrap();

    // Both should reference the same message but different users
    assert_eq!(found_info1.text_message_id, text_message.id);
    assert_eq!(found_info2.text_message_id, text_message.id);
    assert_eq!(found_info1.user_id, recipient1.id);
    assert_eq!(found_info2.user_id, recipient2.id);

    // IDs should be different
    assert_ne!(found_info1.id, found_info2.id);

    // Cleanup
    common::cleanup_text_message_info(info1_id).await;
    common::cleanup_text_message_info(info2_id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient1.id).await;
    common::cleanup_user(recipient2.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_user_id_and_message_id_with_negative_id() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    let negative_id = -1;

    // Act
    let result = repository.find_info_by_user_id_and_message_id(negative_id, negative_id).await;

    // Assert
    assert!(result.is_err(), "Expected error for negative ID");
    // Should return RowNotFound since no record exists with negative ID
    matches!(result.unwrap_err(), sqlx::Error::RowNotFound);
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_user_id_and_message_id_zero_id() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    let zero_id = 0;

    // Act
    let result = repository.find_info_by_user_id_and_message_id(zero_id, zero_id).await;

    // Assert
    assert!(result.is_err(), "Expected error for zero ID");
    // Should return RowNotFound since IDs typically start from 1
    matches!(result.unwrap_err(), sqlx::Error::RowNotFound);
}
