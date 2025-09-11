use crate::common::factories::*;
use crate::common::TestFactory;
use ruggine_client_ui::types::message::{
    Message, MessagePage, PaginationMetadata, TextMessageCreateRequest,
    TextMessageReadDto, PaginatedTextMessageResponse, PaginationMetadataDto,
    TextMessageInfoReadDto, TextMessageInfoReadAtDtoUpdate,
};
use chrono::{DateTime, Utc};

// =============================================================================
// MESSAGE TYPE TESTS
// =============================================================================

#[cfg(test)]
mod message_types_tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let message = TestFactory::mock_message("test");
        
        assert!(message.id > 0);
        assert!(!message.content.is_empty());
        assert!(message.sender_id > 0);
        assert!(message.group_chat_id > 0);
        assert!(message.sent_at <= Utc::now());
    }

    #[test]
    fn test_message_with_custom_params() {
        let message = TestFactory::mock_message_with_params(
            100,
            "Custom message content",
            50,
            25
        );
        
        assert_eq!(message.id, 100);
        assert_eq!(message.content, "Custom message content");
        assert_eq!(message.sender_id, 25);
        assert_eq!(message.group_chat_id, 50);
    }

    #[test]
    fn test_text_message_create_request() {
        let request = TestFactory::mock_message_create_request("Test message", 10);
        
        assert!(!request.content.is_empty());
        assert_eq!(request.group_chat_id, 10);
        assert!(request.content.contains("Test message"));
    }

    #[test]
    fn test_message_read_dto() {
        let dto = TestFactory::mock_message_read_dto("test");
        
        assert!(dto.id > 0);
        assert!(!dto.content.is_empty());
        assert!(dto.sender_id > 0);
        assert!(dto.group_chat_id > 0);
        assert!(!dto.sent_at.is_empty());
        
        // Verify date format is parseable
        let _parsed_date: DateTime<Utc> = dto.sent_at.parse().expect("Should be valid ISO 8601 format");
    }

    #[test]
    fn test_message_conversion_from_dto() {
        let dto = TestFactory::mock_message_read_dto("conversion");
        let original_id = dto.id;
        let original_content = dto.content.clone();
        
        let message: Message = dto.into();
        
        assert_eq!(message.id, original_id);
        assert_eq!(message.content, original_content);
        assert!(message.sent_at <= Utc::now());
    }

    #[test]
    fn test_multiple_messages_have_unique_ids() {
        let messages = TestFactory::mock_multiple_messages(5, "unique", 1);
        
        assert_eq!(messages.len(), 5);
        
        // Check all messages have unique IDs
        let mut ids: Vec<i32> = messages.iter().map(|m| m.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 5, "All message IDs should be unique");
        
        // Check all belong to same group
        assert!(messages.iter().all(|m| m.group_chat_id == 1));
    }

    #[test]
    fn test_message_info_read_dto() {
        let info = TestFactory::mock_message_info_read_dto(100, 50);
        
        assert!(info.id > 0);
        assert_eq!(info.text_message_id, 100);
        assert_eq!(info.user_id, 50);
        assert!(info.sent_at.is_some());
        assert!(info.read_at.is_none()); // Default to unread
        
        if let Some(sent_at) = &info.sent_at {
            let _parsed: DateTime<Utc> = sent_at.parse().expect("Should be valid date");
        }
    }

    #[test]
    fn test_read_update_request() {
        let update = TestFactory::mock_read_update_request(200);
        
        assert_eq!(update.text_message_id, 200);
    }

    #[test]
    fn test_multiple_read_updates() {
        let message_ids = vec![1, 2, 3, 4, 5];
        let updates = TestFactory::mock_multiple_read_updates(&message_ids);
        
        assert_eq!(updates.len(), 5);
        
        for (i, update) in updates.iter().enumerate() {
            assert_eq!(update.text_message_id, message_ids[i]);
        }
    }
}

// =============================================================================
// MESSAGE PAGE AND PAGINATION TESTS
// =============================================================================

#[cfg(test)]
mod pagination_tests {
    use super::*;

    #[test]
    fn test_paginated_message_response() {
        let response = TestFactory::mock_paginated_message_response("page", 10, true);
        
        assert_eq!(response.data.len(), 10);
        assert!(response.pagination.has_more);
        assert!(response.pagination.next_cursor.is_some());
        assert_eq!(response.pagination.page_size, 10);
        assert_eq!(response.pagination.total_count, Some(10));
        
        // Verify all messages have sequential IDs
        for (i, message) in response.data.iter().enumerate() {
            assert_eq!(message.id, (i + 1) as i32);
        }
    }

    #[test]
    fn test_paginated_response_no_more_pages() {
        let response = TestFactory::mock_paginated_message_response("final", 5, false);
        
        assert_eq!(response.data.len(), 5);
        assert!(!response.pagination.has_more);
        assert!(response.pagination.next_cursor.is_none());
        assert_eq!(response.pagination.page_size, 5);
    }

    #[test]
    fn test_message_page_conversion() {
        let response = TestFactory::mock_paginated_message_response("convert", 3, true);
        let original_has_more = response.pagination.has_more;
        let original_page_size = response.pagination.page_size;
        
        let page: MessagePage = response.into();
        
        assert_eq!(page.data.len(), 3);
        assert_eq!(page.pagination.has_more, original_has_more);
        assert_eq!(page.pagination.page_size, original_page_size);
        assert!(page.pagination.next_cursor.is_some());
        
        // Verify converted messages maintain properties
        for (i, message) in page.data.iter().enumerate() {
            assert_eq!(message.id, (i + 1) as i32);
            assert!(message.content.contains("convert"));
        }
    }

    #[test]
    fn test_message_page_direct_creation() {
        let page = TestFactory::mock_message_page("direct", 7, false);
        
        assert_eq!(page.data.len(), 7);
        assert!(!page.pagination.has_more);
        assert!(page.pagination.next_cursor.is_none());
        assert_eq!(page.pagination.page_size, 7);
        
        // Check pagination metadata types
        assert!(matches!(page.pagination.next_cursor, None));
        assert!(page.pagination.total_count.is_some());
    }

    #[test]
    fn test_empty_message_page() {
        let page = TestFactory::mock_message_page("empty", 0, false);
        
        assert!(page.data.is_empty());
        assert!(!page.pagination.has_more);
        assert!(page.pagination.next_cursor.is_none());
        assert_eq!(page.pagination.page_size, 0);
    }
}

// =============================================================================
// MESSAGE BATCH PROCESSING TESTS
// =============================================================================

#[cfg(test)]
mod batch_processing_tests {
    use super::*;

    #[test]
    fn test_batch_size_limits() {
        // Test small batch
        let small_batch = TestFactory::mock_multiple_messages(5, "small", 1);
        assert_eq!(small_batch.len(), 5);
        
        // Test medium batch (typical size)
        let medium_batch = TestFactory::mock_multiple_messages(16, "medium", 1);
        assert_eq!(medium_batch.len(), 16);
        
        // Test large batch
        let large_batch = TestFactory::mock_multiple_messages(50, "large", 1);
        assert_eq!(large_batch.len(), 50);
        
        // Verify that within each batch, IDs are unique
        let mut small_ids: Vec<i32> = small_batch.iter().map(|m| m.id).collect();
        small_ids.sort();
        small_ids.dedup();
        assert_eq!(small_ids.len(), 5, "Small batch IDs should be unique");
        
        let mut medium_ids: Vec<i32> = medium_batch.iter().map(|m| m.id).collect();
        medium_ids.sort();
        medium_ids.dedup();
        assert_eq!(medium_ids.len(), 16, "Medium batch IDs should be unique");
        
        let mut large_ids: Vec<i32> = large_batch.iter().map(|m| m.id).collect();
        large_ids.sort();
        large_ids.dedup();
        assert_eq!(large_ids.len(), 50, "Large batch IDs should be unique");
        
        // Verify each batch has sequential IDs
        for (i, message) in small_batch.iter().enumerate() {
            assert_eq!(message.id, (i + 1) as i32);
        }
        for (i, message) in medium_batch.iter().enumerate() {
            assert_eq!(message.id, (i + 1) as i32);
        }
        for (i, message) in large_batch.iter().enumerate() {
            assert_eq!(message.id, (i + 1) as i32);
        }
    }

    #[test]
    fn test_read_update_batch_creation() {
        let message_ids: Vec<i32> = (1..=20).collect();
        let updates = TestFactory::mock_multiple_read_updates(&message_ids);
        
        assert_eq!(updates.len(), 20);
        
        // Verify all updates have correct message IDs
        for (i, update) in updates.iter().enumerate() {
            assert_eq!(update.text_message_id, message_ids[i]);
        }
        
        // Verify all message IDs are correctly set
        for update in &updates {
            assert!(update.text_message_id > 0, "Message ID should be positive");
        }
    }

    #[test]
    fn test_concurrent_batch_creation() {
        // Simulate concurrent batch creation
        let batch1 = TestFactory::mock_multiple_messages(10, "concurrent1", 1);
        let batch2 = TestFactory::mock_multiple_messages(10, "concurrent2", 2);
        let batch3 = TestFactory::mock_multiple_messages(10, "concurrent3", 3);
        
        // Verify different group IDs
        assert!(batch1.iter().all(|m| m.group_chat_id == 1));
        assert!(batch2.iter().all(|m| m.group_chat_id == 2));
        assert!(batch3.iter().all(|m| m.group_chat_id == 3));
        
        // Verify sequential IDs within each batch
        for (i, message) in batch1.iter().enumerate() {
            assert_eq!(message.id, (i + 1) as i32);
        }
        for (i, message) in batch2.iter().enumerate() {
            assert_eq!(message.id, (i + 1) as i32);
        }
        for (i, message) in batch3.iter().enumerate() {
            assert_eq!(message.id, (i + 1) as i32);
        }
        
        // Verify all batches have correct sizes
        assert_eq!(batch1.len(), 10);
        assert_eq!(batch2.len(), 10);
        assert_eq!(batch3.len(), 10);
    }

    #[test]
    fn test_empty_batch_handling() {
        let empty_batch = TestFactory::mock_multiple_messages(0, "empty", 1);
        assert!(empty_batch.is_empty());
        
        let empty_updates = TestFactory::mock_multiple_read_updates(&[]);
        assert!(empty_updates.is_empty());
    }
}

// =============================================================================
// MESSAGE CONTENT AND FORMATTING TESTS
// =============================================================================

#[cfg(test)]
mod content_tests {
    use super::*;

    #[test]
    fn test_message_content_uniqueness() {
        let messages = TestFactory::mock_multiple_messages(10, "content", 1);
        
        // Collect all content strings
        let contents: Vec<&String> = messages.iter().map(|m| &m.content).collect();
        
        // Verify all contents are unique
        let mut unique_contents = contents.clone();
        unique_contents.sort();
        unique_contents.dedup();
        assert_eq!(unique_contents.len(), contents.len(), "All message contents should be unique");
        
        // Verify all contain expected prefix
        assert!(messages.iter().all(|m| m.content.contains("content")));
    }

    #[test]
    fn test_message_content_with_special_characters() {
        let special_message = TestFactory::mock_message_with_params(
            1,
            "Message with émojis 🚀 and special chars: @#$%^&*()_+-={}[]|\\:;\"'<>,.?/",
            1,
            1
        );
        
        assert!(special_message.content.contains("émojis"));
        assert!(special_message.content.contains("🚀"));
        assert!(special_message.content.contains("@#$%^&*()"));
    }

    #[test]
    fn test_long_message_content() {
        let long_content = "A".repeat(1000);
        let long_message = TestFactory::mock_message_with_params(
            1,
            &long_content,
            1,
            1
        );
        
        assert_eq!(long_message.content.len(), 1000);
        assert!(long_message.content.chars().all(|c| c == 'A'));
    }

    #[test]
    fn test_empty_message_content() {
        let empty_message = TestFactory::mock_message_with_params(
            1,
            "",
            1,
            1
        );
        
        assert!(empty_message.content.is_empty());
    }

    #[test]
    fn test_message_content_with_newlines() {
        let multiline_content = "Line 1\nLine 2\nLine 3\n\nLine 5";
        let multiline_message = TestFactory::mock_message_with_params(
            1,
            multiline_content,
            1,
            1
        );
        
        assert_eq!(multiline_message.content, multiline_content);
        assert_eq!(multiline_message.content.lines().count(), 5);
    }
}

// =============================================================================
// TIMESTAMP AND DATE HANDLING TESTS
// =============================================================================

#[cfg(test)]
mod timestamp_tests {
    use super::*;

    #[test]
    fn test_message_timestamp_validity() {
        let message = TestFactory::mock_message("timestamp");
        let now = Utc::now();
        
        // Message timestamp should be very recent (within last few seconds)
        let diff = now.signed_duration_since(message.sent_at);
        assert!(diff.num_seconds() < 10, "Message timestamp should be very recent");
    }

    #[test]
    fn test_dto_timestamp_format() {
        let dto = TestFactory::mock_message_read_dto("dto_time");
        
        // Verify ISO 8601 format
        let parsed: DateTime<Utc> = dto.sent_at
            .parse()
            .expect("DTO timestamp should be valid ISO 8601 format");
        
        let now = Utc::now();
        let diff = now.signed_duration_since(parsed);
        assert!(diff.num_seconds() < 10);
    }

    #[test]
    fn test_read_update_timestamp_format() {
        let update = TestFactory::mock_read_update_request(1);
        
        // Verify message ID is set correctly
        assert_eq!(update.text_message_id, 1);
    }

    #[test]
    fn test_chronological_message_order() {
        let messages = TestFactory::mock_multiple_messages(5, "chrono", 1);
        
        // Check that messages are in chronological order (or very close)
        for i in 1..messages.len() {
            let prev_time = messages[i - 1].sent_at;
            let curr_time = messages[i].sent_at;
            
            // Allow small time differences due to rapid creation
            let diff = curr_time.signed_duration_since(prev_time);
            assert!(diff.num_milliseconds() >= 0, "Messages should be in chronological order");
        }
    }

    #[test]
    fn test_pagination_cursor_timestamp() {
        let response = TestFactory::mock_paginated_message_response("cursor", 3, true);
        
        if let Some(cursor) = response.pagination.next_cursor {
            let parsed: DateTime<Utc> = cursor
                .parse()
                .expect("Pagination cursor should be valid timestamp");
            
            let now = Utc::now();
            let diff = now.signed_duration_since(parsed);
            assert!(diff.num_seconds() < 10);
        }
    }
}

// =============================================================================
// ERROR HANDLING AND EDGE CASES
// =============================================================================

#[cfg(test)]
mod edge_cases_tests {
    use super::*;

    #[test]
    fn test_message_with_zero_ids() {
        let message = TestFactory::mock_message_with_params(0, "Zero ID test", 0, 0);
        
        assert_eq!(message.id, 0);
        assert_eq!(message.sender_id, 0);
        assert_eq!(message.group_chat_id, 0);
        assert_eq!(message.content, "Zero ID test");
    }

    #[test]
    fn test_message_with_negative_ids() {
        let message = TestFactory::mock_message_with_params(-1, "Negative ID test", -5, -10);
        
        assert_eq!(message.id, -1);
        assert_eq!(message.sender_id, -10);
        assert_eq!(message.group_chat_id, -5);
    }

    #[test]
    fn test_invalid_timestamp_conversion() {
        let mut dto = TestFactory::mock_message_read_dto("invalid");
        dto.sent_at = "invalid-timestamp".to_string();
        
        // Conversion should handle invalid timestamps gracefully
        let message: Message = dto.into();
        
        // Should fallback to current time
        let now = Utc::now();
        let diff = now.signed_duration_since(message.sent_at);
        assert!(diff.num_seconds() < 10, "Should fallback to current time on invalid timestamp");
    }

    #[test]
    fn test_pagination_with_very_large_numbers() {
        let response = PaginatedTextMessageResponse {
            data: vec![],
            pagination: PaginationMetadataDto {
                has_more: true,
                next_cursor: Some("2099-12-31T23:59:59.999Z".to_string()),
                page_size: i32::MAX,
                total_count: Some(i32::MAX),
            },
        };
        
        let page: MessagePage = response.into();
        
        assert_eq!(page.pagination.page_size, i32::MAX);
        assert_eq!(page.pagination.total_count, Some(i32::MAX));
        assert!(page.pagination.next_cursor.is_some());
    }

    #[test]
    fn test_message_equality() {
        let message1 = TestFactory::mock_message_with_params(1, "Test", 1, 1);
        let mut message2 = message1.clone();
        
        assert_eq!(message1, message2);
        
        message2.content = "Different".to_string();
        assert_ne!(message1, message2);
    }

    #[test]
    fn test_message_serialization() {
        let message = TestFactory::mock_message("serialize");
        
        // Test serialization to JSON
        let json = serde_json::to_string(&message).expect("Should serialize to JSON");
        assert!(!json.is_empty());
        assert!(json.contains(&message.content));
        
        // Test deserialization from JSON
        let deserialized: Message = serde_json::from_str(&json).expect("Should deserialize from JSON");
        assert_eq!(message, deserialized);
    }
}
