use crate::{common, mark_message_as_sent_and_read};
use ruggine_server::factory::text_message_factory::TextMessageFactory;
use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_message_id_success_multiple() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("message_id_sender").await;
    let (recipient1, _) = common::create_test_user("message_id_recipient1").await;
    let (recipient2, _) = common::create_test_user("message_id_recipient2").await;
    let group_chat = common::create_test_group_chat("message_id_group", sender_user.id).await;
    
    // Create a text message
    let text_message = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Test message for multiple recipients".to_string())
    ).await;
    
    // Create multiple message infos for the same message
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

    // Act
    let result = repository.find_info_by_message_id(text_message.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find text message infos by message_id: {:?}", result.err());
    let infos = result.unwrap();
    
    assert_eq!(infos.len(), 2, "Expected exactly 2 infos for the message");
    
    // Verify both infos reference the same message
    for info in &infos {
        assert_eq!(info.text_message_id, text_message.id);
    }
    
    // Verify we have infos for both recipients
    let user_ids: Vec<i32> = infos.iter().map(|info| info.user_id).collect();
    assert!(user_ids.contains(&recipient1.id), "Missing info for recipient1");
    assert!(user_ids.contains(&recipient2.id), "Missing info for recipient2");

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
async fn test_find_info_by_message_id_success_single() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("single_info_sender").await;
    let (recipient_user, _) = common::create_test_user("single_info_recipient").await;
    let group_chat = common::create_test_group_chat("single_info_group", sender_user.id).await;
    
    // Create a text message
    let text_message = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Test message for single recipient".to_string())
    ).await;
    
    // Create single message info
    let info = TextMessageFactory::fake_new_text_message_info_with_ids(
        recipient_user.id,
        text_message.id
    );
    
    let info_id = repository.insert_text_message_info(info).await.unwrap();

    // Act
    let result = repository.find_info_by_message_id(text_message.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find text message info by message_id");
    let infos = result.unwrap();
    
    assert_eq!(infos.len(), 1, "Expected exactly 1 info for the message");
    assert_eq!(infos[0].text_message_id, text_message.id);
    assert_eq!(infos[0].user_id, recipient_user.id);

    // Cleanup
    common::cleanup_text_message_info(info_id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_message_id_empty_list() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("empty_list_sender").await;
    let group_chat = common::create_test_group_chat("empty_list_group", sender_user.id).await;
    
    // Create a text message but no message infos
    let text_message = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Message with no infos".to_string())
    ).await;

    // Act
    let result = repository.find_info_by_message_id(text_message.id).await;

    // Assert
    assert!(result.is_ok(), "Expected empty result but got error");
    let infos = result.unwrap();
    assert!(infos.is_empty(), "Expected empty list for message with no infos");

    // Cleanup
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_message_id_nonexistent_message() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    let non_existent_message_id = 999999;

    // Act
    let result = repository.find_info_by_message_id(non_existent_message_id).await;

    // Assert
    assert!(result.is_ok(), "Expected empty result for non-existent message");
    let infos = result.unwrap();
    assert!(infos.is_empty(), "Expected empty list for non-existent message");
}

#[tokio_shared_rt::test(shared)]
async fn test_find_info_by_message_id_with_read_and_unread() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    
    // Create test users and group chat
    let (sender_user, _) = common::create_test_user("mixed_status_sender").await;
    let (recipient1, _) = common::create_test_user("mixed_status_recipient1").await;
    let (recipient2, _) = common::create_test_user("mixed_status_recipient2").await;
    let group_chat = common::create_test_group_chat("mixed_status_group", sender_user.id).await;
    
    // Create a text message
    let text_message = common::create_test_text_message(
        sender_user.id,
        group_chat.id,
        Some("Message with mixed read status".to_string())
    ).await;
    
    // Create message infos - one read, one unread
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
    
    // Mark one as read
    let now = chrono::Utc::now();
    mark_message_as_sent_and_read(info1_id, now).await;

    // Act
    let result = repository.find_info_by_message_id(text_message.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find text message infos");
    let infos = result.unwrap();
    
    assert_eq!(infos.len(), 2, "Expected exactly 2 infos");
    
    // Find the read and unread infos
    let read_info = infos.iter().find(|info| info.read_at.is_some()).unwrap();
    let unread_info = infos.iter().find(|info| info.read_at.is_none()).unwrap();
    
    assert_eq!(read_info.user_id, recipient1.id);
    assert_eq!(unread_info.user_id, recipient2.id);

    // Cleanup
    common::cleanup_text_message_info(info1_id).await;
    common::cleanup_text_message_info(info2_id).await;
    common::cleanup_text_message(text_message.id).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
    common::cleanup_user(recipient1.id).await;
    common::cleanup_user(recipient2.id).await;
}
