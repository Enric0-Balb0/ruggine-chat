use crate::{common, mark_message_as_sent};
use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};
use ruggine_server::factory::text_message_factory::TextMessageFactory;

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_sent_success() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("last_sent_sender").await;
    let (recipient_user, _) = common::create_test_user("last_sent_recipient").await;
    let group_chat = common::create_test_group_chat("last_sent_group", sender_user.id).await;
    
    // Create multiple text messages
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
    let message3 = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Third message".to_string())
    ).await;
    
    // Create message infos with automatic sent_at timestamps
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
    mark_message_as_sent(info1_id, chrono::Utc::now()).await;
    let info2_id = repository.insert_text_message_info(info2).await.unwrap();
    mark_message_as_sent(info2_id, chrono::Utc::now()).await;
    let info3_id = repository.insert_text_message_info(info3).await.unwrap();
    mark_message_as_sent(info3_id, chrono::Utc::now()).await;

    // Act
    let result = repository.find_info_last_sent(recipient_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find last sent info: {:?}", result.err());
    let last_sent_info = result.unwrap().unwrap();
    
    assert_eq!(last_sent_info.user_id, recipient_user.id);
    assert!(last_sent_info.sent_at.is_some(), "Last sent info should have sent_at timestamp");
    
    // Should be one of the messages (typically the most recent one due to timing)
    let message_ids = vec![message1.id, message2.id, message3.id];
    assert!(message_ids.contains(&last_sent_info.text_message_id), 
           "Should return one of the created messages");

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
async fn test_find_info_last_sent_single_message() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("single_sent_sender").await;
    let (recipient_user, _) = common::create_test_user("single_sent_recipient").await;
    let group_chat = common::create_test_group_chat("single_sent_group", sender_user.id).await;
    
    // Create single text message
    let text_message = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Only message".to_string())
    ).await;
    
    // Create message info (sent_at will be set automatically on insert)
    let info = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user.id,
        text_message.id
    );
    
    let info_id = repository.insert_text_message_info(info).await.unwrap();
    mark_message_as_sent(info_id, chrono::Utc::now()).await;

    // Act
    let result = repository.find_info_last_sent(recipient_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find last sent info for single message");
    let last_sent_info = result.unwrap().unwrap();
    
    assert_eq!(last_sent_info.user_id, recipient_user.id);
    assert_eq!(last_sent_info.text_message_id, text_message.id);
    assert!(last_sent_info.sent_at.is_some());

    // Cleanup
    common::cleanup_text_message_info(info_id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_sent_no_messages() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat but no messages
    let (sender_user, _) = common::create_test_user("no_messages_sender").await;
    let (recipient_user, _) = common::create_test_user("no_messages_recipient").await;
    let group_chat = common::create_test_group_chat("no_messages_group", sender_user.id).await;

    // Act
    let result = repository.find_info_last_sent(recipient_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Expected okay result when no messages exist");
    assert!(result.unwrap().is_none());

    // Cleanup
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_sent_user_not_in_group() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("not_in_group_sender").await;
    let (other_user, _) = common::create_test_user("not_in_group_other").await;
    let group_chat = common::create_test_group_chat("not_in_group_chat", sender_user.id).await;

    // Act - Try to find last sent for user not in the group
    let result = repository.find_info_last_sent(other_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Expected success empty result for non-existent group");
    assert!(result.unwrap().is_none());

    // Cleanup
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(other_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_sent_nonexistent_group() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    let (user, _) = common::create_test_user("nonexistent_group_user").await;
    let non_existent_group_id = 999999;

    // Act
    let result = repository.find_info_last_sent(user.id, non_existent_group_id).await;

    // Assert
    assert!(result.is_ok(), "Expected success empty result for non-existent group");
    assert!(result.unwrap().is_none());

    // Cleanup
    common::cleanup_user(user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_sent_different_users_same_group() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("multi_user_sender").await;
    let (recipient1, _) = common::create_test_user("multi_user_recipient1").await;
    let (recipient2, _) = common::create_test_user("multi_user_recipient2").await;
    let group_chat = common::create_test_group_chat("multi_user_group", sender_user.id).await;
    
    // Create text messages
    let message1 = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Message for recipient1".to_string())
    ).await;
    let message2 = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Message for recipient2".to_string())
    ).await;
    
    // Create infos for both users
    let info1 = TextMessageFactory::fake_new_text_message_info_with_ids(recipient1.id, message1.id);
    let info2 = TextMessageFactory::fake_new_text_message_info_with_ids(recipient2.id, message2.id);
    
    let info1_id = repository.insert_text_message_info(info1).await.unwrap();
    mark_message_as_sent(info1_id, chrono::Utc::now()).await;
    let info2_id = repository.insert_text_message_info(info2).await.unwrap();
    mark_message_as_sent(info2_id, chrono::Utc::now()).await;

    // Act - Get last sent for each user
    let result1 = repository.find_info_last_sent(recipient1.id, group_chat.id).await;
    let result2 = repository.find_info_last_sent(recipient2.id, group_chat.id).await;

    // Assert
    assert!(result1.is_ok(), "Failed to find last sent for recipient1");
    assert!(result2.is_ok(), "Failed to find last sent for recipient2");
    
    let last_sent1 = result1.unwrap().unwrap();
    let last_sent2 = result2.unwrap().unwrap();
    
    assert_eq!(last_sent1.user_id, recipient1.id);
    assert_eq!(last_sent1.text_message_id, message1.id);
    
    assert_eq!(last_sent2.user_id, recipient2.id);
    assert_eq!(last_sent2.text_message_id, message2.id);
    
    assert!(last_sent1.sent_at.is_some());
    assert!(last_sent2.sent_at.is_some());

    // Cleanup
    common::cleanup_text_message_info(info1_id).await;
    common::cleanup_text_message_info(info2_id).await;
    common::cleanup_text_message(message1.id).await;
    common::cleanup_text_message(message2.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient1.id).await;
    common::cleanup_user(recipient2.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_last_sent_with_chronological_order() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("chrono_sender").await;
    let (recipient_user, _) = common::create_test_user("chrono_recipient").await;
    let group_chat = common::create_test_group_chat("chrono_group", sender_user.id).await;
    
    // Create multiple text messages with slight delays to ensure different timestamps
    let message1 = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("First message".to_string())
    ).await;
    
    // Small delay to ensure different timestamps
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    let message2 = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Second message".to_string())
    ).await;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    
    let message3 = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Third message".to_string())
    ).await;
    
    // Create message infos in order
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
    mark_message_as_sent(info1_id, chrono::Utc::now()).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let info2_id = repository.insert_text_message_info(info2).await.unwrap();
    mark_message_as_sent(info2_id, chrono::Utc::now()).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    let info3_id = repository.insert_text_message_info(info3).await.unwrap();
    mark_message_as_sent(info3_id, chrono::Utc::now()).await;

    // Act
    let result = repository.find_info_last_sent(recipient_user.id, group_chat.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find last sent info");
    let last_sent_info = result.unwrap().unwrap();
    
    assert_eq!(last_sent_info.user_id, recipient_user.id);
    // Should return the most recently sent (last inserted) message
    assert_eq!(last_sent_info.text_message_id, message3.id, "Should return the most recently sent message");
    assert!(last_sent_info.sent_at.is_some());

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
