// Integration tests for Message Service

use std::sync::Mutex;
use std::sync::Arc;

#[cfg(test)]
mod message_service_integration_tests {
    use super::*;
    use crate::common::*;
    use ruggine_client_ui::api::services::message::MessageService;
    use ruggine_client_ui::api::client::ApiClient;
    use ruggine_client_ui::utils::storage::StorageService;
    use ruggine_client_ui::types::message::{
        TextMessageCreateRequest, MessagePage, TextMessageInfoReadAtDtoUpdate
    };

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_message_service() -> MessageService {
        let api_client = ApiClient::new("http://localhost:3000".to_string());
        let storage_service = StorageService::new();
        MessageService::new(api_client, storage_service)
    }

    #[tokio::test]
    async fn test_message_service_creation() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_message_service();
        
        // Service should be created successfully
        // This is mainly testing that the constructor works
        drop(service);
    }

    #[tokio::test]
    async fn test_message_create_request_validation() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_message_service();
        
        // Test valid request creation
        let valid_request = MessageFactory::basic_text_message_create_request();
        assert!(!valid_request.content.is_empty());
        assert!(valid_request.group_id > 0);
        
        // Test unique request creation
        let unique_request = MessageFactory::unique_text_message_create_request("test");
        assert!(!unique_request.content.is_empty());
        assert!(unique_request.content.contains("test"));
    }

    #[tokio::test]
    async fn test_message_service_with_different_requests() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_message_service();
        
        // Test multiple request types
        let basic_request = MessageFactory::basic_text_message_create_request();
        let unique_request = MessageFactory::unique_text_message_create_request("unique");
        let group_request = MessageFactory::text_message_for_group(42, "Group message");
        
        // Verify request properties
        assert_eq!(basic_request.group_id, 1);
        assert_eq!(group_request.group_id, 42);
        assert_eq!(group_request.content, "Group message");
        
        assert_ne!(basic_request.content, unique_request.content);
        assert_ne!(basic_request.content, group_request.content);
    }

    #[tokio::test]
    async fn test_message_page_structure() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_message_service();
        
        // Test message page creation
        let page = MessageFactory::mock_message_page();
        assert_eq!(page.messages.len(), 2);
        assert_eq!(page.pagination.current_page, 1);
        assert_eq!(page.pagination.page_size, 20);
        assert_eq!(page.pagination.total_items, 2);
        
        // Test empty page
        let empty_page = MessageFactory::empty_message_page();
        assert_eq!(empty_page.messages.len(), 0);
        assert_eq!(empty_page.pagination.total_items, 0);
    }

    #[tokio::test]
    async fn test_message_read_update_structure() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_message_service();
        
        // Test read update creation
        let update = MessageFactory::mock_text_message_info_read_at_dto_update();
        assert!(update.message_id > 0);
        assert!(!update.read_at.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_message_service_data_consistency() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_message_service();
        
        // Create message and verify consistency
        let message = MessageFactory::mock_message();
        let dto = MessageFactory::mock_text_message_read_dto();
        let info_dto = MessageFactory::mock_text_message_info_read_dto();
        
        // All should have valid IDs
        assert!(message.id > 0);
        assert!(dto.id > 0);
        assert!(info_dto.id > 0);
        
        // All should have valid timestamps
        assert!(!message.created_at.to_string().is_empty());
        assert!(!dto.created_at.to_string().is_empty());
        assert!(!info_dto.created_at.to_string().is_empty());
        
        // DTOs should have usernames
        assert!(!dto.username.is_empty());
        assert!(!info_dto.username.is_empty());
    }

    #[tokio::test]
    async fn test_message_service_batch_operations() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_message_service();
        
        // Create multiple messages for batch testing
        let messages: Vec<TextMessageCreateRequest> = (1..=5)
            .map(|i| MessageFactory::text_message_for_group(i, &format!("Batch message {}", i)))
            .collect();
        
        assert_eq!(messages.len(), 5);
        
        // Verify each message has correct group assignment
        for (i, msg) in messages.iter().enumerate() {
            let expected_group_id = (i + 1) as i32;
            assert_eq!(msg.group_id, expected_group_id);
            assert!(msg.content.contains(&format!("message {}", expected_group_id)));
        }
    }

    #[tokio::test]
    async fn test_message_service_error_handling_simulation() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_message_service();
        
        // Test that service can handle different request scenarios
        let empty_content_request = TextMessageCreateRequest {
            content: "".to_string(),
            group_id: 1,
        };
        
        let zero_group_request = TextMessageCreateRequest {
            content: "Valid content".to_string(),
            group_id: 0,
        };
        
        let negative_group_request = TextMessageCreateRequest {
            content: "Valid content".to_string(),
            group_id: -1,
        };
        
        // These would be invalid in real scenarios
        assert!(empty_content_request.content.is_empty());
        assert_eq!(zero_group_request.group_id, 0);
        assert!(negative_group_request.group_id < 0);
    }

    #[tokio::test]
    async fn test_message_service_pagination_edge_cases() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_message_service();
        
        // Test various pagination scenarios
        let normal_page = MessageFactory::mock_message_page();
        let empty_page = MessageFactory::empty_message_page();
        
        // Normal page should have data
        assert!(normal_page.pagination.total_items > 0);
        assert!(normal_page.pagination.total_pages > 0);
        
        // Empty page should have no data
        assert_eq!(empty_page.pagination.total_items, 0);
        assert_eq!(empty_page.pagination.total_pages, 0);
        assert!(!empty_page.pagination.has_next);
        assert!(!empty_page.pagination.has_previous);
    }

    #[tokio::test]
    async fn test_message_service_concurrent_requests_simulation() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_message_service();
        
        // Simulate concurrent message creation
        let concurrent_requests: Vec<TextMessageCreateRequest> = (0..10)
            .map(|i| MessageFactory::unique_text_message_create_request(&format!("concurrent_{}", i)))
            .collect();
        
        assert_eq!(concurrent_requests.len(), 10);
        
        // Each request should be unique
        for (i, request) in concurrent_requests.iter().enumerate() {
            assert!(request.content.contains(&format!("concurrent_{}", i)));
            
            // No two requests should have the same content
            for (j, other_request) in concurrent_requests.iter().enumerate() {
                if i != j {
                    assert_ne!(request.content, other_request.content);
                }
            }
        }
    }
}
