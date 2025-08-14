use ruggine_server::repository::text_message_repository::{TextMessageRepository};
use ruggine_server::service::text_message_service::{TextMessageService, TextMessageServiceTrait};
use ruggine_server::dto::text_message_pagination_dto::TextMessagePaginationQuery;
use crate::common;
use std::sync::Arc;

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_paginated_first_page() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    let service = TextMessageService::new(Arc::new(repository));

    let (sender_user, _) = common::create_test_user("paginated_sender").await;
    let group_chat = common::create_test_group_chat("paginated_group", sender_user.id).await;
    
    // Create multiple messages for the group
    let messages = common::create_test_text_messages_for_group(group_chat.id, sender_user.id, 5).await;
    let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();

    // Act
    let pagination_query = TextMessagePaginationQuery {
        cursor: None,
        limit: 10,
    };
    let result = service.find_by_group_chat_id_paginated(group_chat.id, pagination_query).await;

    // Assert
    assert!(result.is_ok(), "Failed to find paginated messages: {:?}", result.err());
    let paginated_response = result.unwrap();
    
    assert_eq!(paginated_response.data.len(), 5, "Should return all 5 messages");
    assert_eq!(paginated_response.pagination.has_more, false, "Should not have more pages");
    assert!(paginated_response.pagination.next_cursor.is_none(), "Should not have next cursor");
    
    // Verify all messages belong to the correct group
    for message_dto in &paginated_response.data {
        assert_eq!(message_dto.group_chat_id, group_chat.id);
        assert_eq!(message_dto.sender_id, sender_user.id);
    }
    
    // Messages should be ordered by sent_at DESC (newest first)
    for i in 0..paginated_response.data.len()-1 {
        assert!(paginated_response.data[i].sent_at >= paginated_response.data[i+1].sent_at);
    }

    // Cleanup
    common::cleanup_text_messages(message_ids).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user_by_id(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_paginated_with_cursor() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    let service = TextMessageService::new(Arc::new(repository));

    let (sender_user, _) = common::create_test_user("cursor_sender").await;
    let group_chat = common::create_test_group_chat("cursor_group", sender_user.id).await;
    
    // Create messages with some delay to ensure different timestamps
    let messages = common::create_test_text_messages_for_group(group_chat.id, sender_user.id, 10).await;
    let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();

    // Get first page
    let first_page_query = TextMessagePaginationQuery {
        cursor: None,
        limit: 5,
    };
    let first_page_result = service.find_by_group_chat_id_paginated(group_chat.id, first_page_query).await;
    assert!(first_page_result.is_ok());
    let first_page = first_page_result.unwrap();
    
    assert_eq!(first_page.data.len(), 5);
    assert_eq!(first_page.pagination.has_more, true);
    assert!(first_page.pagination.next_cursor.is_some());

    // Use the next cursor for the second page
    let cursor_string = first_page.pagination.next_cursor.unwrap();
    let cursor_datetime = chrono::DateTime::parse_from_rfc3339(&cursor_string)
        .unwrap()
        .with_timezone(&chrono::Utc);

    // Act - Get second page with cursor
    let second_page_query = TextMessagePaginationQuery {
        cursor: Some(cursor_datetime),
        limit: 5,
    };
    let second_page_result = service.find_by_group_chat_id_paginated(group_chat.id, second_page_query).await;

    // Assert
    assert!(second_page_result.is_ok(), "Failed to find paginated messages with cursor");
    let second_page = second_page_result.unwrap();
    
    assert_eq!(second_page.data.len(), 5, "Should return remaining 5 messages");
    assert_eq!(second_page.pagination.has_more, false, "Should not have more pages");
    
    // All messages in second page should be older than those in first page
    if let (Some(last_first_page), Some(first_second_page)) = (
        first_page.data.last(),
        second_page.data.first()
    ) {
        assert!(last_first_page.sent_at > first_second_page.sent_at, 
                "Second page messages should be older than first page messages");
    }

    // Verify all messages belong to the correct group
    for message_dto in &second_page.data {
        assert_eq!(message_dto.group_chat_id, group_chat.id);
        assert_eq!(message_dto.sender_id, sender_user.id);
    }

    // Cleanup
    common::cleanup_text_messages(message_ids).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user_by_id(sender_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_paginated_empty_group() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    let service = TextMessageService::new(Arc::new(repository));

    let (creator_user, _) = common::create_test_user("empty_creator").await;
    let group_chat = common::create_test_group_chat("empty_group", creator_user.id).await;

    // Act - Try to get messages from empty group
    let pagination_query = TextMessagePaginationQuery {
        cursor: None,
        limit: 10,
    };
    let result = service.find_by_group_chat_id_paginated(group_chat.id, pagination_query).await;

    // Assert
    assert!(result.is_ok(), "Should succeed even for empty group");
    let paginated_response = result.unwrap();
    
    assert_eq!(paginated_response.data.len(), 0, "Should return empty data for group with no messages");
    assert_eq!(paginated_response.pagination.has_more, false, "Should not have more pages");
    assert!(paginated_response.pagination.next_cursor.is_none(), "Should not have next cursor");

    // Cleanup
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user_by_id(creator_user.id).await;
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_paginated_multi_sender() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    let service = TextMessageService::new(Arc::new(repository));

    // Create multiple users and a group
    let users = common::create_test_users("multi_sender", 3).await;
    let creator_id = users[0].0.id;
    let group_chat = common::create_test_group_chat("multi_sender_group", creator_id).await;
    
    // Create messages from different senders
    let sender_ids: Vec<i32> = users.iter().map(|(user, _)| user.id).collect();
    let messages = common::create_test_text_messages_multi_sender(group_chat.id, sender_ids.clone()).await;
    let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();

    // Act
    let pagination_query = TextMessagePaginationQuery {
        cursor: None,
        limit: 10,
    };
    let result = service.find_by_group_chat_id_paginated(group_chat.id, pagination_query).await;

    // Assert
    assert!(result.is_ok(), "Failed to find messages from multiple senders");
    let paginated_response = result.unwrap();
    
    assert_eq!(paginated_response.data.len(), 3, "Should return messages from all senders");
    assert_eq!(paginated_response.pagination.has_more, false, "Should not have more pages");
    
    // Verify all sender IDs are present
    let found_sender_ids: std::collections::HashSet<i32> = paginated_response.data.iter().map(|m| m.sender_id).collect();
    let expected_sender_ids: std::collections::HashSet<i32> = sender_ids.iter().cloned().collect();
    assert_eq!(found_sender_ids, expected_sender_ids, "Should have messages from all expected senders");
    
    // Verify all messages belong to the correct group
    for message_dto in &paginated_response.data {
        assert_eq!(message_dto.group_chat_id, group_chat.id);
    }

    // Cleanup
    common::cleanup_text_messages(message_ids).await;
    common::cleanup_group_chat(group_chat.id).await;
    for (user, _) in users {
        common::cleanup_user_by_id(user.id).await;
    }
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_paginated_non_existing_group() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    let service = TextMessageService::new(Arc::new(repository));
    
    let non_existing_group_id = 999999;

    // Act
    let pagination_query = TextMessagePaginationQuery {
        cursor: None,
        limit: 10,
    };
    let result = service.find_by_group_chat_id_paginated(non_existing_group_id, pagination_query).await;

    // Assert - Should return empty result, not an error (group may exist but have no messages)
    assert!(result.is_ok(), "Should succeed even for non-existing group");
    let paginated_response = result.unwrap();
    assert_eq!(paginated_response.data.len(), 0, "Should return empty data for non-existing group");
}

#[tokio_shared_rt::test(shared)]
async fn test_find_by_group_chat_id_paginated_limit_handling() {
    // Arrange
    let db = common::get_database().await;
    let repository = TextMessageRepository::new(&db);
    let service = TextMessageService::new(Arc::new(repository));

    let (sender_user, _) = common::create_test_user("limit_sender").await;
    let group_chat = common::create_test_group_chat("limit_group", sender_user.id).await;
    
    // Create 7 messages
    let messages = common::create_test_text_messages_for_group(group_chat.id, sender_user.id, 7).await;
    let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();

    // Act with limit of 3
    let pagination_query = TextMessagePaginationQuery {
        cursor: None,
        limit: 3,
    };
    let result = service.find_by_group_chat_id_paginated(group_chat.id, pagination_query).await;

    // Assert
    assert!(result.is_ok());
    let paginated_response = result.unwrap();
    
    assert_eq!(paginated_response.data.len(), 3, "Should respect the limit");
    assert_eq!(paginated_response.pagination.has_more, true, "Should indicate more pages available");
    assert!(paginated_response.pagination.next_cursor.is_some(), "Should provide next cursor");
    assert_eq!(paginated_response.pagination.page_size, 3, "Should return correct page size");

    // Cleanup
    common::cleanup_text_messages(message_ids).await;
    common::cleanup_group_chat(group_chat.id).await;
    common::cleanup_user_by_id(sender_user.id).await;
}
