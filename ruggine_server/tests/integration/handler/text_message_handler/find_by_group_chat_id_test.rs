use ruggine_server::handler::text_message_handler::find_by_group_chat_id::find_by_group_chat_id;
use ruggine_server::dto::text_message_pagination_dto::TextMessagePaginationQuery;
use chrono::{Utc, TimeZone};
use axum::{Extension, extract::{Path, Query, State}};
use crate::common::{
    cleanup_text_messages, create_test_text_messages_for_group_without_message_info, create_test_user,
    create_test_group_chat, cleanup_group_chat, cleanup_user, create_text_message_state
};

#[cfg(test)]
mod find_by_group_chat_id_handler_integration_tests {
    use ruggine_server::error::api_error::ApiError;
    use ruggine_server::error::request_error::ValidatedQuery;
    use ruggine_server::error::text_message_error::TextMessageError;
    use crate::{cleanup_test_user_from_a_group_chat, create_test_group_chat_with_invitation_and_membership, test_user_leave_from_a_group};
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_unauthorized_user() {
        // Arrange: Create user who is NOT member of the group
        let (unauthorized_user, _) = create_test_user("find_msgs_handler_unauthorized").await;
        let (owner_user, _) = create_test_user("find_msgs_handler_owner").await;
        let group = create_test_group_chat("find_msgs_handler_unauthorized", owner_user.id).await;
        
        // Create some messages (but user is not authorized to see them)
        let _messages = create_test_text_messages_for_group_without_message_info(group.id, owner_user.id, 3).await;
        
        let state = create_text_message_state().await;
        let pagination_query = TextMessagePaginationQuery {
            cursor: None,
            limit: 10,
        };

        // Act - try to access with unauthorized user
        let result = find_by_group_chat_id(
            Extension(unauthorized_user.clone()),
            State(state),
            Path(group.id),
            ValidatedQuery(pagination_query),
        ).await;

        // Assert
        assert!(result.is_err(), "Handler should return error for unauthorized user");

        // Cleanup
        let message_ids: Vec<i32> = _messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_group_chat(group.id).await;
        cleanup_user(unauthorized_user.id).await;
        cleanup_user(owner_user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_existing_messages() {
        // Arrange: Create user, group and messages
        let (user, _) = create_test_user("find_msgs_handler_existing").await;
        let group = create_test_group_chat_with_invitation_and_membership("find_msgs_handler_existing", user.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, user.id, 5).await;
        
        let state = create_text_message_state().await;
        let pagination_query = TextMessagePaginationQuery {
            cursor: None,
            limit: 10,
        };

        // Act
        let result = find_by_group_chat_id(
            Extension(user.clone()),
            State(state),
            Path(group.id),
            ValidatedQuery(pagination_query),
        ).await;

        // Assert
        assert!(result.is_ok(), "Handler should successfully find messages");
        let response = result.unwrap().0;
        
        assert_eq!(response.data.len(), 5);
        assert!(!response.pagination.next_cursor.is_some());
        
        // Verify messages are sorted by sent_at in descending order
        for i in 1..response.data.len() {
            assert!(
                response.data[i - 1].sent_at >= response.data[i].sent_at,
                "Messages should be ordered by sent_at DESC"
            );
        }

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_text_messages(message_ids).await;
        cleanup_group_chat(group.id).await;
        cleanup_user(user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_with_pagination_limit() {
        // Arrange: Create user, group and more messages than limit
        let (user, _) = create_test_user("find_msgs_handler_paginated").await;
        let group = create_test_group_chat_with_invitation_and_membership("find_msgs_handler_paginated", user.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, user.id, 15).await;
        
        let state = create_text_message_state().await;
        let pagination_query = TextMessagePaginationQuery {
            cursor: None,
            limit: 10,
        };

        // Act
        let result = find_by_group_chat_id(
            Extension(user.clone()),
            State(state),
            Path(group.id),
            ValidatedQuery(pagination_query),
        ).await;

        // Assert
        assert!(result.is_ok(), "Handler should successfully paginate messages");
        let response = result.unwrap().0;
        
        assert_eq!(response.data.len(), 10);
        assert!(response.pagination.next_cursor.is_some());

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user(user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_with_cursor() {
        // Arrange: Create user, group and messages
        let (user, _) = create_test_user("find_msgs_handler_cursor").await;
        let group = create_test_group_chat_with_invitation_and_membership("find_msgs_handler_cursor", user.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, user.id, 10).await;
        
        // Get a cursor timestamp from one of the messages (middle one)
        let cursor_time = messages[5].sent_at;
        
        let state = create_text_message_state().await;
        let pagination_query = TextMessagePaginationQuery {
            cursor: Some(cursor_time),
            limit: 5,
        };

        // Act
        let result = find_by_group_chat_id(
            Extension(user.clone()),
            State(state),
            Path(group.id),
            ValidatedQuery(pagination_query),
        ).await;

        // Assert
        assert!(result.is_ok(), "Handler should handle cursor pagination");
        let response = result.unwrap().0;
        
        assert!(response.data.len() <= 5);
        
        // Verify all returned messages are older than cursor
        for message in &response.data {
            assert!(message.sent_at < cursor_time, "Messages should be older than cursor");
        }

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user(user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_empty_group() {
        // Arrange: Create user and empty group
        let (user, _) = create_test_user("find_msgs_handler_empty").await;
        let group = create_test_group_chat_with_invitation_and_membership("find_msgs_handler_empty", user.id).await;
        
        let state = create_text_message_state().await;
        let pagination_query = TextMessagePaginationQuery {
            cursor: None,
            limit: 10,
        };

        // Act
        let result = find_by_group_chat_id(
            Extension(user.clone()),
            State(state),
            Path(group.id),
            ValidatedQuery(pagination_query),
        ).await;

        // Assert
        assert!(result.is_ok(), "Handler should handle empty groups");
        let response = result.unwrap().0;
        
        assert!(response.data.is_empty());
        assert!(response.pagination.next_cursor.is_none());

        // Cleanup
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user(user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_non_existing_group() {
        // Arrange: Create user for authentication
        let (user, _) = create_test_user("find_msgs_handler_nonexisting").await;
        let state = create_text_message_state().await;
        let pagination_query = TextMessagePaginationQuery {
            cursor: None,
            limit: 10,
        };

        let non_existing_group_id = 999999;

        // Act
        let result = find_by_group_chat_id(
            Extension(user.clone()),
            State(state),
            Path(non_existing_group_id),
            ValidatedQuery(pagination_query),
        ).await;

        // Assert - Should return error
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::TextMessageError(TextMessageError::GroupChatNotFound)));

        // Cleanup
        cleanup_user(user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_custom_limit() {
        // Arrange: Create user, group and messages
        let (user, _) = create_test_user("find_msgs_handler_custom_limit").await;
        let group = create_test_group_chat_with_invitation_and_membership("find_msgs_handler_custom_limit", user.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, user.id, 8).await;
        
        let state = create_text_message_state().await;
        let pagination_query = TextMessagePaginationQuery {
            cursor: None,
            limit: 3,
        };

        // Act
        let result = find_by_group_chat_id(
            Extension(user.clone()),
            State(state),
            Path(group.id),
            ValidatedQuery(pagination_query),
        ).await;

        // Assert
        assert!(result.is_ok(), "Handler should respect custom limit");
        let response = result.unwrap().0;
        
        assert_eq!(response.data.len(), 3);
        assert!(response.pagination.next_cursor.is_some());

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user(user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_multi_sender() {
        // Arrange: Create multiple users and a group
        let (creator_user, _) = create_test_user("handler_multi_sender_creator").await;
        let group = create_test_group_chat_with_invitation_and_membership("handler_multi_sender_group", creator_user.id).await;
        
        // Create additional users for the group
        use crate::common::{create_test_users_for_a_group, create_test_text_messages_multi_sender_without_message_info, cleanup_test_users_from_a_group_chat, cleanup_test_users};
        let users = create_test_users_for_a_group("handler_multi_sender", 3, &group).await;
        
        // Create messages from different senders
        let sender_ids: Vec<i32> = users.iter().map(|(user, _, _)| user.id).collect();
        let messages = create_test_text_messages_multi_sender_without_message_info(group.id, sender_ids.clone()).await;
        
        let state = create_text_message_state().await;
        let pagination_query = TextMessagePaginationQuery {
            cursor: None,
            limit: 10,
        };

        // Act - use creator as authenticated user (they have membership as creator)
        let result = find_by_group_chat_id(
            Extension(creator_user.clone()),
            State(state),
            Path(group.id),
            ValidatedQuery(pagination_query),
        ).await;

        // Assert
        assert!(result.is_ok(), "Handler should handle messages from multiple senders");
        let response = result.unwrap().0;
        
        assert_eq!(response.data.len(), 3, "Should return messages from all senders");
        assert!(!response.pagination.has_more, "Should not have more pages");
        
        // Verify all sender IDs are present
        let found_sender_ids: std::collections::HashSet<i32> = response.data.iter().map(|m| m.sender_id).collect();
        let expected_sender_ids: std::collections::HashSet<i32> = sender_ids.iter().cloned().collect();
        assert_eq!(found_sender_ids, expected_sender_ids, "Should have messages from all expected senders");
        
        // Verify all messages belong to the correct group
        for message_dto in &response.data {
            assert_eq!(message_dto.group_chat_id, group.id);
        }

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_users_from_a_group_chat(sender_ids.clone(), group.id).await;
        cleanup_test_user_from_a_group_chat(creator_user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(sender_ids).await;
        cleanup_user(creator_user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_comprehensive_pagination() {
        // Arrange: Create user, group and enough messages for multi-page pagination
        let (user, _) = create_test_user("handler_comprehensive_pagination").await;
        let group = create_test_group_chat_with_invitation_and_membership("handler_comprehensive_pagination", user.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, user.id, 25).await;
        
        let state = create_text_message_state().await;
        
        // First page
        let first_page_query = TextMessagePaginationQuery {
            cursor: None,
            limit: 10,
        };

        let first_result = find_by_group_chat_id(
            Extension(user.clone()),
            State(state.clone()),
            Path(group.id),
            ValidatedQuery(first_page_query),
        ).await;

        assert!(first_result.is_ok(), "Handler should handle first page");
        let first_response = first_result.unwrap().0;
        
        assert_eq!(first_response.data.len(), 10);
        assert!(first_response.pagination.has_more);
        assert!(first_response.pagination.next_cursor.is_some());
        
        // Second page using cursor
        let cursor = first_response.pagination.next_cursor.unwrap();
        let second_page_query = TextMessagePaginationQuery {
            cursor: Some(cursor),
            limit: 10,
        };

        let second_result = find_by_group_chat_id(
            Extension(user.clone()),
            State(state.clone()),
            Path(group.id),
            ValidatedQuery(second_page_query),
        ).await;

        // Act & Assert for second page
        assert!(second_result.is_ok(), "Handler should handle second page with cursor");
        let second_response = second_result.unwrap().0;
        
        assert_eq!(second_response.data.len(), 10);
        assert!(second_response.pagination.has_more);
        
        // Messages in second page should be older than those in first page
        if let (Some(last_first_page), Some(first_second_page)) = (
            first_response.data.last(),
            second_response.data.first()
        ) {
            assert!(last_first_page.sent_at > first_second_page.sent_at, 
                    "Second page messages should be older than first page messages");
        }

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user(user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_pagination_boundary_conditions() {
        // Arrange: Create user, group and exactly enough messages to test boundary conditions
        let (user, _) = create_test_user("handler_boundary_conditions").await;
        let group = create_test_group_chat_with_invitation_and_membership("handler_boundary_conditions", user.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, user.id, 10).await;
        
        let state = create_text_message_state().await;

        // Test: Exact limit match
        let exact_limit_query = TextMessagePaginationQuery {
            cursor: None,
            limit: 10,
        };

        let exact_result = find_by_group_chat_id(
            Extension(user.clone()),
            State(state.clone()),
            Path(group.id),
            ValidatedQuery(exact_limit_query),
        ).await;

        assert!(exact_result.is_ok());
        let exact_response = exact_result.unwrap().0;
        assert_eq!(exact_response.data.len(), 10);
        assert!(!exact_response.pagination.has_more, "Should not have more when exact match");
        assert!(exact_response.pagination.next_cursor.is_none());

        // Test: Limit larger than available messages
        let large_limit_query = TextMessagePaginationQuery {
            cursor: None,
            limit: 20,
        };

        let large_result = find_by_group_chat_id(
            Extension(user.clone()),
            State(state.clone()),
            Path(group.id),
            ValidatedQuery(large_limit_query),
        ).await;

        assert!(large_result.is_ok());
        let large_response = large_result.unwrap().0;
        assert_eq!(large_response.data.len(), 10, "Should return all available messages");
        assert!(!large_response.pagination.has_more);

        // Test: Small limit (1)
        let small_limit_query = TextMessagePaginationQuery {
            cursor: None,
            limit: 1,
        };

        let small_result = find_by_group_chat_id(
            Extension(user.clone()),
            State(state),
            Path(group.id),
            ValidatedQuery(small_limit_query),
        ).await;

        assert!(small_result.is_ok());
        let small_response = small_result.unwrap().0;
        assert_eq!(small_response.data.len(), 1);
        assert!(small_response.pagination.has_more);
        assert!(small_response.pagination.next_cursor.is_some());

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user(user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_user_left_group() {
        // Arrange: Create user and group with membership, then user leaves
        let (user, _) = create_test_user("find_msgs_handler_left").await;
        let group = create_test_group_chat_with_invitation_and_membership("find_msgs_handler_left", user.id).await;
        let messages = create_test_text_messages_for_group_without_message_info(group.id, user.id, 3).await;
        
        let state = create_text_message_state().await;
        
        // User leaves the group (making membership inactive)
        test_user_leave_from_a_group(user.id, group.id).await;
        
        let pagination_query = TextMessagePaginationQuery {
            cursor: None,
            limit: 10,
        };

        // Act - try to access messages after leaving group
        let result = find_by_group_chat_id(
            Extension(user.clone()),
            State(state),
            Path(group.id),
            ValidatedQuery(pagination_query),
        ).await;

        // Assert
        assert!(result.is_err(), "Handler should return error for user who left group");
        
        if let Err(error) = result {
            match error {
                ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
                    // This is the expected error
                }
                _ => panic!("Expected TextMessageError::Forbidden, got: {:?}", error),
            }
        }

        // Cleanup
        let message_ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        cleanup_text_messages(message_ids).await;
        cleanup_test_user_from_a_group_chat(user.id, group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_user(user.id).await;
    }
}
