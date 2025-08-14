use crate::common;
use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};
use ruggine_server::factory::text_message_factory::TextMessageFactory;

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_paginated_integration_first_page() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    let (sender_user, _) = common::create_test_user("paginated_sender").await;
    let group_chat = common::create_test_group_chat("paginated_group", sender_user.id).await;

    // Create multiple messages for the group
    let messages = common::create_test_text_messages_for_group(group_chat.id, sender_user.id, 5).await;
    let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();

    // Act
    let result = repository.find_by_group_chat_id_paginated(group_chat.id, None, 10).await;

    // Assert
    assert!(result.is_ok(), "Failed to find paginated messages: {:?}", result.err());
    let found_messages = result.unwrap();

    assert_eq!(found_messages.len(), 5, "Should return all 5 messages");
    assert!(found_messages.iter().all(|m| m.group_chat_id == group_chat.id));

    // Messages should be ordered by sent_at DESC (newest first)
    for i in 0..found_messages.len()-1 {
        assert!(found_messages[i].sent_at >= found_messages[i+1].sent_at);
    }

    // Cleanup
    common::cleanup_text_messages(message_ids).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user_by_id(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_paginated_integration_with_cursor() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    let (sender_user, _) = common::create_test_user("cursor_sender").await;
    let group_chat = common::create_test_group_chat("cursor_group", sender_user.id).await;

    // Create messages with some delay to ensure different timestamps
    let messages = common::create_test_text_messages_for_group(group_chat.id, sender_user.id, 10).await;
    let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();

    // Get first page
    let first_page = repository.find_by_group_chat_id_paginated(group_chat.id, None, 5).await.unwrap();
    assert_eq!(first_page.len(), 5);

    // Use the timestamp of the last message as cursor for next page
    let cursor = first_page.last().unwrap().sent_at;

    // Act - Get second page with cursor
    let result = repository.find_by_group_chat_id_paginated(group_chat.id, Some(cursor), 5).await;

    // Assert
    assert!(result.is_ok(), "Failed to find paginated messages with cursor");
    let second_page = result.unwrap();

    assert_eq!(second_page.len(), 5, "Should return remaining 5 messages");
    assert!(second_page.iter().all(|m| m.group_chat_id == group_chat.id));

    // All messages in second page should be older than the cursor
    let cursor_time = first_page.last().unwrap().sent_at;
    assert!(second_page.iter().all(|m| m.sent_at < cursor_time));

    // Cleanup
    common::cleanup_text_messages(message_ids).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user_by_id(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_paginated_integration_empty_group() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    let (creator_user, _) = common::create_test_user("empty_creator").await;
    let group_chat = common::create_test_group_chat("empty_group", creator_user.id).await;

    // Act - Try to get messages from empty group
    let result = repository.find_by_group_chat_id_paginated(group_chat.id, None, 10).await;

    // Assert
    assert!(result.is_ok(), "Should succeed even for empty group");
    let messages = result.unwrap();
    assert_eq!(messages.len(), 0, "Should return empty vector for group with no messages");

    // Cleanup
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user_by_id(creator_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_paginated_integration_invalid_cursor() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    let (sender_user, _) = common::create_test_user("invalid_cursor_sender").await;
    let group_chat = common::create_test_group_chat("invalid_cursor_group", sender_user.id).await;

    let invalid_cursor = chrono::Utc::now() - chrono::Duration::days(1);

    // Act
    let result = repository.find_by_group_chat_id_paginated(group_chat.id, Some(invalid_cursor), 10).await;

    // Assert
    assert!(result.is_ok(), "Should succeed with valid datetime cursor");

    // Cleanup
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user_by_id(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_paginated_integration_multi_sender() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);

    // Create multiple users and a group
    let users = common::create_test_users("multi_sender", 3).await;
    let creator_id = users[0].0.id;
    let group_chat = common::create_test_group_chat("multi_sender_group", creator_id).await;

    // Create messages from different senders
    let sender_ids: Vec<i32> = users.iter().map(|(user, _)| user.id).collect();
    let messages = common::create_test_text_messages_multi_sender(group_chat.id, sender_ids.clone()).await;
    let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();

    // Act
    let result = repository.find_by_group_chat_id_paginated(group_chat.id, None, 10).await;

    // Assert
    assert!(result.is_ok(), "Failed to find messages from multiple senders");
    let found_messages = result.unwrap();

    assert_eq!(found_messages.len(), 3, "Should return messages from all senders");
    assert!(found_messages.iter().all(|m| m.group_chat_id == group_chat.id));

    // Verify all sender IDs are present
    let found_sender_ids: std::collections::HashSet<i32> = found_messages.iter().map(|m| m.sender_id).collect();
    let expected_sender_ids: std::collections::HashSet<i32> = sender_ids.iter().cloned().collect();
    assert_eq!(found_sender_ids, expected_sender_ids, "Should have messages from all expected senders");

    // Cleanup
    common::cleanup_text_messages(message_ids).await;
    common::cleanup_group_chat(group_chat.id).await;
    for (user, _) in users {
        common::cleanup_user_by_id(user.id).await;
    }
}
