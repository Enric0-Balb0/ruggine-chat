use ruggine_server::entity::text_message::TextMessageInfoUpdate;
use crate::{common, mark_message_as_sent_and_read};
use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};
use ruggine_server::factory::text_message_factory::TextMessageFactory;

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_read_success() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("last_read_sender").await;
    let (recipient_user, _) = common::create_test_user("last_read_recipient").await;
    let group_chat = common::create_test_group_chat("last_read_group", sender_user.id).await;
    
    // Create multiple text messages
    let message1 = common::create_test_text_message_without_message_info(
        sender_user.id,
        group_chat.id,
        Some("First message".to_string())
    ).await;
    let message2 = common::create_test_text_message_without_message_info(
        sender_user.id,
        group_chat.id,
        Some("Second message".to_string())
    ).await;
    let message3 = common::create_test_text_message_without_message_info(
        sender_user.id,
        group_chat.id,
        Some("Third message".to_string())
    ).await;
    
    // Create message infos
    let info1 = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user.id,
        message1.id
    );
    let info2 = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user.id,
        message2.id
    );
    let info3 = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user.id,
        message3.id
    );
    
    let info1_id = repository.insert_text_message_info(info1).await.unwrap();
    let info2_id = repository.insert_text_message_info(info2).await.unwrap();
    let info3_id = repository.insert_text_message_info(info3).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Mark messages as read with different timestamps
    let base_time = chrono::Utc::now();
    let read_time1 = base_time - chrono::Duration::milliseconds(10);
    let read_time2 = base_time - chrono::Duration::milliseconds(5);

    // Leave message3 unread
    repository.update_info(TextMessageInfoUpdate {
        id: info1_id,
        sent_at: Some(read_time1),
        read_at: Some(read_time1),
    }).await.unwrap();
    repository.update_info(TextMessageInfoUpdate {
        id: info2_id,
        sent_at: Some(read_time2),
        read_at: Some(read_time2),
    }).await.unwrap();

    // Act
    let result = repository.find_info_last_read(recipient_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find last read info: {:?}", result.err());
    let last_read_info = result.unwrap().unwrap();
    
    assert_eq!(last_read_info.user_id, recipient_user.id);
    assert_eq!(last_read_info.text_message_id, message2.id, "Should return the most recently read message");
    assert!(last_read_info.read_at.is_some(), "Last read info should have read_at timestamp");

    // Cleanup
    common::cleanup_text_message_info(info1_id).await;
    common::cleanup_text_message_info(info2_id).await;
    common::cleanup_text_message_info(info3_id).await;
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_text_message(message3.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_read_single_message() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("single_read_sender").await;
    let (recipient_user, _) = common::create_test_user("single_read_recipient").await;
    let group_chat = common::create_test_group_chat("single_read_group", sender_user.id).await;
    
    // Create single text message
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id,
        group_chat.id,
        Some("Only message".to_string())
    ).await;
    
    // Create message info and mark as read
    let info = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user.id,
        text_message.id
    );
    
    let info_id = repository.insert_text_message_info(info).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    let read_time = chrono::Utc::now();
    repository.update_info(TextMessageInfoUpdate {
        id: info_id,
        sent_at: Some(read_time),
        read_at: Some(read_time),
    }).await.unwrap();

    // Act
    let result = repository.find_info_last_read(recipient_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find last read info for single message");
    let last_read_info = result.unwrap().unwrap();
    
    assert_eq!(last_read_info.user_id, recipient_user.id);
    assert_eq!(last_read_info.text_message_id, text_message.id);
    assert!(last_read_info.read_at.is_some());

    // Cleanup
    common::cleanup_text_message_info(info_id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_read_no_read_messages() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("no_read_sender").await;
    let (recipient_user, _) = common::create_test_user("no_read_recipient").await;
    let group_chat = common::create_test_group_chat("no_read_group", sender_user.id).await;
    
    // Create text message but don't mark as read
    let text_message = common::create_test_text_message_without_message_info(
        sender_user.id,
        group_chat.id,
        Some("Unread message".to_string())
    ).await;
    
    let info = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user.id,
        text_message.id
    );
    
    let info_id = repository.insert_text_message_info(info).await.unwrap();

    // Act
    let result = repository.find_info_last_read(recipient_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Expected okay result when no messages exist");
    assert!(result.unwrap().is_none());

    // Cleanup
    common::cleanup_text_message_info(info_id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_read_user_not_in_group() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("not_in_group_sender").await;
    let (other_user, _) = common::create_test_user("not_in_group_other").await;
    let group_chat = common::create_test_group_chat("not_in_group_chat", sender_user.id).await;

    // Act - Try to find last read for user not in the group
    let result = repository.find_info_last_read(other_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Expected success empty result for non-existent group");
    assert!(result.unwrap().is_none());

    // Cleanup
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(other_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_read_nonexistent_group() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    let (user, _) = common::create_test_user("nonexistent_group_user").await;
    let non_existent_group_id = 999999;

    // Act
    let result = repository.find_info_last_read(user.id, non_existent_group_id).await;

    // Assert
    assert!(result.is_ok(), "Expected success empty result for non-existent group");
    assert!(result.unwrap().is_none());

    // Cleanup
    common::cleanup_user(user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_read_different_users_same_group() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("multi_user_sender").await;
    let (recipient1, _) = common::create_test_user("multi_user_recipient1").await;
    let (recipient2, _) = common::create_test_user("multi_user_recipient2").await;
    let group_chat = common::create_test_group_chat("multi_user_group", sender_user.id).await;
    
    // Create text messages
    let message1 = common::create_test_text_message_without_message_info(
        sender_user.id,
        group_chat.id,
        Some("Message for recipient1".to_string())
    ).await;
    let message2 = common::create_test_text_message_without_message_info(
        sender_user.id,
        group_chat.id,
        Some("Message for recipient2".to_string())
    ).await;
    
    // Create infos for both users
    let info1_1 = TextMessageFactory::fake_new_text_message_info_with_ids(recipient1.id, message1.id);
    let info1_2 = TextMessageFactory::fake_new_text_message_info_with_ids(recipient1.id, message2.id);
    let info2_1 = TextMessageFactory::fake_new_text_message_info_with_ids(recipient2.id, message1.id);
    let info2_2 = TextMessageFactory::fake_new_text_message_info_with_ids(recipient2.id, message2.id);
    
    let info1_1_id = repository.insert_text_message_info(info1_1).await.unwrap();
    let info1_2_id = repository.insert_text_message_info(info1_2).await.unwrap();
    let info2_1_id = repository.insert_text_message_info(info2_1).await.unwrap();
    let info2_2_id = repository.insert_text_message_info(info2_2).await.unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Mark different messages as read for each user
    let base_time = chrono::Utc::now();
    repository.update_info(TextMessageInfoUpdate {
        id: info1_1_id,
        sent_at: Some(base_time - chrono::Duration::milliseconds(10)),
        read_at: Some(base_time - chrono::Duration::milliseconds(10)),
    }).await.unwrap();
    repository.update_info(TextMessageInfoUpdate {
        id: info2_2_id,
        sent_at: Some(base_time - chrono::Duration::milliseconds(5)),
        read_at: Some(base_time - chrono::Duration::milliseconds(5)),
    }).await.unwrap();

    // Act - Get last read for each user
    let result1 = repository.find_info_last_read(recipient1.id, group_chat.id).await;
    let result2 = repository.find_info_last_read(recipient2.id, group_chat.id).await;

    // Assert
    assert!(result1.is_ok(), "Failed to find last read for recipient1");
    assert!(result2.is_ok(), "Failed to find last read for recipient2");
    
    let last_read1 = result1.unwrap().unwrap();
    let last_read2 = result2.unwrap().unwrap();
    
    assert_eq!(last_read1.user_id, recipient1.id);
    assert_eq!(last_read1.text_message_id, message1.id);
    
    assert_eq!(last_read2.user_id, recipient2.id);
    assert_eq!(last_read2.text_message_id, message2.id);

    // Cleanup
    common::cleanup_text_message_info(info1_1_id).await;
    common::cleanup_text_message_info(info1_2_id).await;
    common::cleanup_text_message_info(info2_1_id).await;
    common::cleanup_text_message_info(info2_2_id).await;
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient1.id).await;
    common::cleanup_user(recipient2.id).await;
}