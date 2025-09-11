// Message-related test data factory

use super::base_factory::BaseFactory;
use chrono::{DateTime, Utc};
use ruggine_client_ui::types::message::{
    TextMessageCreateRequest, MessagePage, Message, PaginationMetadata,
    TextMessageInfoReadAtDtoUpdate
};

pub struct MessageFactory;

impl MessageFactory {
    /// Create mock message for testing
    pub fn mock_message(prefix: &str) -> Message {
        let unique_id = BaseFactory::get_unique_id();
        let content = BaseFactory::get_unique_message_content(prefix);
        
        Message {
            id: unique_id as i32,
            content,
            sender_id: 1,
            group_chat_id: 1,
            sent_at: chrono::Utc::now(),
        }
    }

    /// Create mock messages for testing with multiple messages
    pub fn mock_messages() -> Vec<Message> {
        let unique_id = BaseFactory::get_unique_id();
        let content = BaseFactory::get_unique_message_content("multiple");
        
        vec![
            Message {
                id: unique_id as i32,
                content: content.clone(),
                sender_id: 1,
                group_chat_id: 1,
                sent_at: chrono::Utc::now(),
            },
            Message {
                id: (unique_id + 1) as i32,
                content,
                sender_id: 1,
                group_chat_id: 1,
                sent_at: chrono::Utc::now(),
            }
        ]
    }

    /// Create unique TextMessageCreateRequest for testing
    pub fn unique_text_message_create_request(prefix: &str) -> TextMessageCreateRequest {
        let content = BaseFactory::get_unique_message_content(prefix);
        
        TextMessageCreateRequest {
            content,
            group_chat_id: 1,
        }
    }
    
    /// Create basic text message create request
    pub fn basic_text_message_create_request() -> TextMessageCreateRequest {
        TextMessageCreateRequest {
            content: "Hello, this is a test message!".to_string(),
            group_chat_id: 1,
        }
    }
    
    /// Create text message request for specific group
    pub fn text_message_for_group(group_chat_id: i32, content: &str) -> TextMessageCreateRequest {
        TextMessageCreateRequest {
            content: content.to_string(),
            group_chat_id,
        }
    }
    
    /// Create message with specific details
    pub fn message_with_details(id: i32, content: &str, sender_id: i32, group_chat_id: i32) -> Message {
        Message {
            id,
            content: content.to_string(),
            sender_id,
            group_chat_id,
            sent_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
        }
    }
    
    /// Create mock message page for pagination testing
    pub fn mock_message_page() -> MessagePage {
        let messages = vec![
            Self::mock_message("page_msg1"),
            Self::mock_message("page_msg2"),
        ];
        
        MessagePage {
            data: messages,
            pagination: PaginationMetadata {
                has_more: true,
                next_cursor: Some(DateTime::from_timestamp(1000000100, 0).unwrap()),
                page_size: 20,
                total_count: Some(50),
            },
        }
    }
    
    /// Create empty message page for testing
    pub fn empty_message_page() -> MessagePage {
        MessagePage {
            data: vec![],
            pagination: PaginationMetadata {
                has_more: false,
                next_cursor: None,
                page_size: 20,
                total_count: Some(0),
            },
        }
    }
    
    /// Create TextMessageInfoReadAtDtoUpdate for testing
    pub fn text_message_read_update(text_message_id: i32) -> TextMessageInfoReadAtDtoUpdate {
        TextMessageInfoReadAtDtoUpdate {
            text_message_id,
        }
    }
    
    /// Create multiple messages for testing
    pub fn multiple_messages(count: usize) -> Vec<Message> {
        (0..count)
            .map(|i| Self::message_with_details(
                i as i32 + 1,
                &format!("Message {}", i + 1),
                (i as i32 % 3) + 1, // Different senders
                1, // Same group
            ))
            .collect()
    }
    
    /// Create uniqueness test messages
    pub fn uniqueness_test_messages() -> Vec<TextMessageCreateRequest> {
        vec![
            Self::text_message_for_group(1, "First message"),
            Self::text_message_for_group(1, "Second message"), 
            Self::text_message_for_group(2, "Third message in different group"),
        ]
    }

    /// Create message with specific parameters
    pub fn mock_message_with_params(message_id: i32, content: &str, group_id: i32, sender_id: i32) -> Message {
        Message {
            id: message_id,
            content: content.to_string(),
            sender_id,
            group_chat_id: group_id,
            sent_at: chrono::Utc::now(),
        }
    }

    /// Create message create request
    pub fn mock_message_create_request(content: &str, group_id: i32) -> TextMessageCreateRequest {
        TextMessageCreateRequest {
            content: content.to_string(),
            group_chat_id: group_id,
        }
    }

    /// Create message read dto
    pub fn mock_message_read_dto(prefix: &str) -> Message {
        Self::mock_message(prefix)
    }

    /// Create multiple messages with parameters
    pub fn mock_multiple_messages(count: usize, prefix: &str, group_id: i32) -> Vec<Message> {
        (0..count)
            .map(|i| Self::mock_message_with_params(
                i as i32 + 1,
                &format!("{} {}", prefix, i + 1),
                group_id,
                1,
            ))
            .collect()
    }

    /// Create message info read dto
    pub fn mock_message_info_read_dto(message_id: i32, read_at_id: i32) -> TextMessageInfoReadAtDtoUpdate {
        TextMessageInfoReadAtDtoUpdate {
            text_message_id: message_id,
        }
    }

    /// Create read update request
    pub fn mock_read_update_request(message_id: i32) -> TextMessageInfoReadAtDtoUpdate {
        Self::text_message_read_update(message_id)
    }

    /// Create multiple read updates
    pub fn mock_multiple_read_updates(message_ids: Vec<i32>) -> Vec<TextMessageInfoReadAtDtoUpdate> {
        message_ids
            .into_iter()
            .enumerate()
            .map(|(i, id)| Self::mock_message_info_read_dto(id, i as i32))
            .collect()
    }

    /// Create paginated message response
    pub fn mock_paginated_message_response(prefix: &str, count: usize, has_more: bool) -> MessagePage {
        let messages = Self::mock_multiple_messages(count, prefix, 1);
        
        MessagePage {
            data: messages,
            pagination: PaginationMetadata {
                has_more,
                next_cursor: if has_more { 
                    Some(chrono::Utc::now()) 
                } else { 
                    None 
                },
                page_size: count as i32,
                total_count: Some(count as i32),
            },
        }
    }
}
