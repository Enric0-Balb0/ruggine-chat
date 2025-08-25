use crate::common;
use chrono::{Duration, Utc};
use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_datetime_range_integration_messages_within_range() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    let (sender_user, _) = common::create_test_user("datetime_range_sender").await;
    let group_chat = common::create_test_group_chat("datetime_range_group", sender_user.id).await;

    // Create messages with specific timestamps
    let base_time = Utc::now();
    let message1 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Message 1".to_string())
    ).await;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let message2 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Message 2".to_string())
    ).await;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let message3 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Message 3".to_string())
    ).await;

    let message_ids = vec![message1.id, message2.id, message3.id];

    // Set time range to include all messages
    let min_datetime = base_time - Duration::minutes(5);
    let max_datetime = base_time + Duration::minutes(5);

    // Act
    let result = repository.find_by_group_chat_id_datetime_range(
        group_chat.id, 
        min_datetime, 
        max_datetime
    ).await;

    // Assert
    assert!(result.is_ok(), "Failed to find messages within datetime range: {:?}", result.err());
    let found_messages = result.unwrap();

    assert_eq!(found_messages.len(), 3, "Should return all 3 messages within range");
    assert!(found_messages.iter().all(|m| m.group_chat_id == group_chat.id));
    assert!(found_messages.iter().all(|m| m.sent_at >= min_datetime && m.sent_at <= max_datetime));

    // Messages should be ordered by sent_at DESC (newest first)
    for i in 0..found_messages.len()-1 {
        assert!(found_messages[i].sent_at >= found_messages[i+1].sent_at);
    }

    // Cleanup
    common::cleanup_text_messages(message_ids).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_datetime_range_integration_partial_range() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    let (sender_user, _) = common::create_test_user("partial_range_sender").await;
    let group_chat = common::create_test_group_chat("partial_range_group", sender_user.id).await;

    // Create messages with different timestamps
    // Message 1: older (outside range)
    let message1 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Old message".to_string())
    ).await;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
    
    // Message 2: within range
    let message2 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Within range message 1".to_string())
    ).await;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Message 3: within range
    let message3 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Within range message 2".to_string())
    ).await;

    let message_ids = vec![message1.id, message2.id, message3.id];

    // Set time range to exclude message1 but include message2 and message3
    let base_time = Utc::now();
    let min_datetime = base_time - Duration::seconds(1);
    let max_datetime = base_time + Duration::minutes(1);

    // Act
    let result = repository.find_by_group_chat_id_datetime_range(
        group_chat.id, 
        min_datetime, 
        max_datetime
    ).await;

    // Assert
    assert!(result.is_ok(), "Failed to find messages within partial datetime range");
    let found_messages = result.unwrap();

    assert_eq!(found_messages.len(), 2, "Should return only 2 messages within range");
    assert!(found_messages.iter().all(|m| m.group_chat_id == group_chat.id));
    assert!(found_messages.iter().all(|m| m.sent_at >= min_datetime && m.sent_at <= max_datetime));

    // Verify that message1 is not included
    assert!(!found_messages.iter().any(|m| m.id == message1.id));
    // Verify that message2 and message3 are included
    assert!(found_messages.iter().any(|m| m.id == message2.id));
    assert!(found_messages.iter().any(|m| m.id == message3.id));

    // Cleanup
    common::cleanup_text_messages(message_ids).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_datetime_range_integration_empty_range() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    let (sender_user, _) = common::create_test_user("empty_range_sender").await;
    let group_chat = common::create_test_group_chat("empty_range_group", sender_user.id).await;

    // Create a message
    let message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message".to_string())
    ).await;

    // Set time range that excludes all messages (in the past)
    let min_datetime = Utc::now() - Duration::hours(2);
    let max_datetime = Utc::now() - Duration::hours(1);

    // Act
    let result = repository.find_by_group_chat_id_datetime_range(
        group_chat.id, 
        min_datetime, 
        max_datetime
    ).await;

    // Assert
    assert!(result.is_ok(), "Should succeed even with empty range");
    let found_messages = result.unwrap();
    assert_eq!(found_messages.len(), 0, "Should return empty vector for range with no messages");

    // Cleanup
    common::cleanup_text_messages(vec![message.id]).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_datetime_range_integration_non_existent_group() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    let non_existent_group_id = 99999;
    let min_datetime = Utc::now() - Duration::hours(1);
    let max_datetime = Utc::now() + Duration::hours(1);

    // Act
    let result = repository.find_by_group_chat_id_datetime_range(
        non_existent_group_id, 
        min_datetime, 
        max_datetime
    ).await;

    // Assert
    assert!(result.is_ok(), "Should succeed for non-existent group");
    let found_messages = result.unwrap();
    assert_eq!(found_messages.len(), 0, "Should return empty vector for non-existent group");
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_datetime_range_integration_exact_boundaries() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    let (sender_user, _) = common::create_test_user("boundary_sender").await;
    let group_chat = common::create_test_group_chat("boundary_group", sender_user.id).await;

    // Create a message
    let message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Boundary test message".to_string())
    ).await;

    // Set exact boundaries using the message timestamp
    let min_datetime = message.sent_at;
    let max_datetime = message.sent_at;

    // Act
    let result = repository.find_by_group_chat_id_datetime_range(
        group_chat.id, 
        min_datetime, 
        max_datetime
    ).await;

    // Assert
    assert!(result.is_ok(), "Should succeed with exact boundary timestamps");
    let found_messages = result.unwrap();
    assert_eq!(found_messages.len(), 1, "Should return the message when boundaries match exactly");
    assert_eq!(found_messages[0].id, message.id);
    assert_eq!(found_messages[0].sent_at, message.sent_at);

    // Cleanup
    common::cleanup_text_messages(vec![message.id]).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_datetime_range_integration_multiple_groups() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    let (sender_user, _) = common::create_test_user("multi_group_sender").await;
    let group_chat1 = common::create_test_group_chat("multi_group_1", sender_user.id).await;
    let group_chat2 = common::create_test_group_chat("multi_group_2", sender_user.id).await;

    // Create messages in both groups
    let message1 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat1.id, 
        Some("Message in group 1".to_string())
    ).await;
    
    let message2 = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat2.id, 
        Some("Message in group 2".to_string())
    ).await;

    let message_ids = vec![message1.id, message2.id];

    // Set time range to include both messages
    let min_datetime = Utc::now() - Duration::minutes(5);
    let max_datetime = Utc::now() + Duration::minutes(5);

    // Act - Query for group1 only
    let result = repository.find_by_group_chat_id_datetime_range(
        group_chat1.id, 
        min_datetime, 
        max_datetime
    ).await;

    // Assert
    assert!(result.is_ok(), "Should succeed for group-specific query");
    let found_messages = result.unwrap();
    assert_eq!(found_messages.len(), 1, "Should return only messages from group1");
    assert_eq!(found_messages[0].group_chat_id, group_chat1.id);
    assert_eq!(found_messages[0].id, message1.id);

    // Cleanup
    common::cleanup_text_messages(message_ids).await;
    common::cleanup_group_chat(group_chat1.id).await;
    common::cleanup_group_chat(group_chat2.id).await;
    common::cleanup_user(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_datetime_range_integration_multiple_senders() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    // Create multiple users and a group
    let users = common::create_test_users("multi_sender_range", 3).await;
    let creator_id = users[0].0.id;
    let group_chat = common::create_test_group_chat("multi_sender_range_group", creator_id).await;

    // Create messages from different senders
    let sender_ids: Vec<i32> = users.iter().map(|(user, _)| user.id).collect();
    let messages = common::create_test_text_messages_multi_sender_without_message_info(
        group_chat.id, 
        sender_ids.clone()
    ).await;
    let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();

    // Set time range to include all messages
    let min_datetime = Utc::now() - Duration::minutes(5);
    let max_datetime = Utc::now() + Duration::minutes(5);

    // Act
    let result = repository.find_by_group_chat_id_datetime_range(
        group_chat.id, 
        min_datetime, 
        max_datetime
    ).await;

    // Assert
    assert!(result.is_ok(), "Failed to find messages from multiple senders within range");
    let found_messages = result.unwrap();

    assert_eq!(found_messages.len(), 3, "Should return messages from all senders");
    assert!(found_messages.iter().all(|m| m.group_chat_id == group_chat.id));
    assert!(found_messages.iter().all(|m| m.sent_at >= min_datetime && m.sent_at <= max_datetime));

    // Verify all sender IDs are present
    let found_sender_ids: std::collections::HashSet<i32> = found_messages.iter().map(|m| m.sender_id).collect();
    let expected_sender_ids: std::collections::HashSet<i32> = sender_ids.into_iter().collect();
    assert_eq!(found_sender_ids, expected_sender_ids, "All senders should be represented");

    // Cleanup
    common::cleanup_text_messages(message_ids).await;
    common::cleanup_group_chat(group_chat.id).await;
    let user_ids: Vec<i32> = users.iter().map(|(user, _)| user.id).collect();
    common::cleanup_test_users(user_ids).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_datetime_range_integration_inverted_range() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    let (sender_user, _) = common::create_test_user("inverted_range_sender").await;
    let group_chat = common::create_test_group_chat("inverted_range_group", sender_user.id).await;

    // Create a message
    let message = common::create_test_text_message_without_message_info(
        sender_user.id, 
        group_chat.id, 
        Some("Test message".to_string())
    ).await;

    // Set inverted time range (max < min)
    let min_datetime = Utc::now() + Duration::hours(1);
    let max_datetime = Utc::now() - Duration::hours(1);

    // Act
    let result = repository.find_by_group_chat_id_datetime_range(
        group_chat.id, 
        min_datetime, 
        max_datetime
    ).await;

    // Assert
    assert!(result.is_ok(), "Should succeed with inverted range");
    let found_messages = result.unwrap();
    assert_eq!(found_messages.len(), 0, "Should return empty vector for inverted range");

    // Cleanup
    common::cleanup_text_messages(vec![message.id]).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user(sender_user.id).await;
}
