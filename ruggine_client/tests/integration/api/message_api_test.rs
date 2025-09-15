// Integration tests for Message API

#[cfg(test)]
mod message_api_integration_tests {
    use crate::common::*;
    

    #[tokio::test]
    async fn test_text_message_create_request_validation() {
        let basic_request = MessageFactory::basic_text_message_create_request();

        assert!(!basic_request.content.is_empty());
        assert!(basic_request.group_chat_id > 0);
    }

    #[tokio::test]
    async fn test_unique_text_message_create_request() {
        let request1 = MessageFactory::unique_text_message_create_request("msg1");
        let request2 = MessageFactory::unique_text_message_create_request("msg2");

        assert_ne!(request1.content, request2.content);
    }

    #[tokio::test]
    async fn test_text_message_for_specific_group() {
        let message = MessageFactory::text_message_for_group(42, "Test content");

        assert_eq!(message.group_chat_id, 42);
        assert_eq!(message.content, "Test content");
    }

    #[tokio::test]
    async fn test_mock_message_creation() {
        let message = MessageFactory::mock_message("test");

        assert!(message.id > 0);
        assert!(!message.content.is_empty());
        assert!(message.sender_id > 0);
        assert!(message.group_chat_id > 0);
        assert!(!message.sent_at.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_message_with_specific_details() {
        let message = MessageFactory::message_with_details(123, "Specific content", 5, 10);

        assert_eq!(message.id, 123);
        assert_eq!(message.content, "Specific content");
        assert_eq!(message.sender_id, 5);
        assert_eq!(message.group_chat_id, 10);
    }

    #[tokio::test]
    async fn test_message_page_creation_and_pagination() {
        let page = MessageFactory::mock_message_page();

        assert_eq!(page.data.len(), 2);
        assert_eq!(page.pagination.page_size, 20);
        assert_eq!(page.pagination.total_count, Some(2));
    }

    #[tokio::test]
    async fn test_empty_message_page() {
        let empty_page = MessageFactory::empty_message_page();

        assert_eq!(empty_page.data.len(), 0);
        assert_eq!(empty_page.pagination.total_count, Some(0));
    }

    #[tokio::test]
    async fn test_paginated_message_response_helper() {
        let response = MessageFactory::mock_paginated_message_response("pfx", 2, false);
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.pagination.total_count, Some(2));
    }

    #[tokio::test]
    async fn test_text_message_read_at_update() {
        let update = MessageFactory::text_message_read_update(1);
        assert!(update.text_message_id > 0);
    }
}
