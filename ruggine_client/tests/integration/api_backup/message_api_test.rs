// Integration tests for Message API

#[cfg(test)]
mod message_api_integration_tests {
    use crate::common::*;
    use ruggine_client_ui::types::message::{
        Message, MessagePage, TextMessageCreateRequest,
        TextMessageReadDto, PaginatedTextMessageResponse,
        TextMessageInfoReadDto, TextMessageInfoReadAtDtoUpdate,
    };

    #[tokio::test]
    async fn test_text_message_create_request_validation() {
        let basic_request = MessageFactory::basic_text_message_create_request();
        
        assert!(!basic_request.content.is_empty());
        assert!(basic_request.group_id > 0);
        assert_eq!(basic_request.content, "Test message content");
        assert_eq!(basic_request.group_id, 1);
    }
    
    #[tokio::test]
    async fn test_unique_text_message_create_request() {
        let request1 = MessageFactory::unique_text_message_create_request("msg1");
        let request2 = MessageFactory::unique_text_message_create_request("msg2");
        
        assert_ne!(request1.content, request2.content);
        assert!(request1.content.contains("msg1"));
        assert!(request2.content.contains("msg2"));
    }
    
    #[tokio::test]
    async fn test_text_message_for_specific_group() {
        let message = MessageFactory::text_message_for_group(42, "Test content");
        
        assert_eq!(message.group_id, 42);
        assert_eq!(message.content, "Test content");
    }
    
    #[tokio::test]
    async fn test_mock_message_creation() {
        let message = MessageFactory::mock_message();
        
        assert!(message.id > 0);
        assert!(!message.content.is_empty());
        assert!(message.user_id > 0);
        assert!(message.group_id > 0);
        assert!(!message.created_at.to_string().is_empty());
        assert!(!message.updated_at.to_string().is_empty());
    }
    
    #[tokio::test]
    async fn test_message_with_specific_details() {
        let message = MessageFactory::message_with_details(
            123, "Specific content", 5, 10
        );
        
        assert_eq!(message.id, 123);
        assert_eq!(message.content, "Specific content");
        assert_eq!(message.user_id, 5);
        assert_eq!(message.group_id, 10);
    }
    
    #[tokio::test]
    async fn test_text_message_read_dto() {
        let message_dto = MessageFactory::mock_text_message_read_dto();
        
        assert!(message_dto.id > 0);
        assert!(!message_dto.content.is_empty());
        assert!(message_dto.user_id > 0);
        assert!(message_dto.group_id > 0);
        assert!(!message_dto.username.is_empty());
        assert_eq!(message_dto.username, "testuser");
    }
    
    #[tokio::test]
    async fn test_message_page_creation() {
        let page = MessageFactory::mock_message_page();
        
        assert_eq!(page.messages.len(), 2);
        assert_eq!(page.pagination.current_page, 1);
        assert_eq!(page.pagination.page_size, 20);
        assert_eq!(page.pagination.total_items, 2);
        assert_eq!(page.pagination.total_pages, 1);
        assert!(!page.pagination.has_next);
        assert!(!page.pagination.has_previous);
    }
    
    #[tokio::test]
    async fn test_empty_message_page() {
        let empty_page = MessageFactory::empty_message_page();
        
        assert_eq!(empty_page.messages.len(), 0);
        assert_eq!(empty_page.pagination.total_items, 0);
        assert_eq!(empty_page.pagination.total_pages, 0);
        assert!(!empty_page.pagination.has_next);
        assert!(!empty_page.pagination.has_previous);
    }
    
    #[tokio::test]
    async fn test_paginated_text_message_response() {
        let response = MessageFactory::mock_paginated_text_message_response();
        
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.pagination.current_page, 1);
        assert_eq!(response.pagination.page_size, 20);
        assert_eq!(response.pagination.total_items, 2);
        assert_eq!(response.pagination.total_pages, 1);
    }
    
    #[tokio::test]
    async fn test_text_message_info_read_dto() {
        let info_dto = MessageFactory::mock_text_message_info_read_dto();
        
        assert!(info_dto.id > 0);
        assert!(!info_dto.content.is_empty());
        assert!(info_dto.user_id > 0);
        assert!(info_dto.group_id > 0);
        assert!(!info_dto.username.is_empty());
        assert!(info_dto.read_at.is_none()); // Should be None initially
    }
    
    #[tokio::test]
    async fn test_text_message_read_at_update() {
        let update_dto = MessageFactory::mock_text_message_info_read_at_dto_update();
        
        assert!(update_dto.message_id > 0);
        assert!(!update_dto.read_at.to_string().is_empty());
    }
    
    #[tokio::test]
    async fn test_message_content_uniqueness() {
        let msg1 = MessageFactory::unique_text_message_create_request("prefix1");
        let msg2 = MessageFactory::unique_text_message_create_request("prefix2");
        let msg3 = MessageFactory::unique_text_message_create_request("prefix1");
        
        assert_ne!(msg1.content, msg2.content);
        assert_ne!(msg1.content, msg3.content);
        assert_ne!(msg2.content, msg3.content);
    }
    
    #[tokio::test]
    async fn test_multiple_messages_for_different_groups() {
        let messages: Vec<TextMessageCreateRequest> = (1..=5)
            .map(|i| MessageFactory::text_message_for_group(i, &format!("Message for group {}", i)))
            .collect();
            
        assert_eq!(messages.len(), 5);
        
        for (i, msg) in messages.iter().enumerate() {
            let expected_group_id = (i + 1) as i32;
            assert_eq!(msg.group_id, expected_group_id);
            assert!(msg.content.contains(&format!("group {}", expected_group_id)));
        }
    }
    
    #[tokio::test]
    async fn test_message_timestamps_consistency() {
        let message = MessageFactory::mock_message();
        
        // created_at should be <= updated_at
        assert!(message.created_at <= message.updated_at);
    }
}
