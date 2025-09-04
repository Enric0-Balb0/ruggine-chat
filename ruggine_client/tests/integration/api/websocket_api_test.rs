// Integration tests for WebSocket API

#[cfg(test)]
mod websocket_api_integration_tests {
    use crate::common::*;
    use ruggine_client_ui::types::message_ws::{
        WebSocketMessage, ClientAction, GroupAction, ServerEvent, GroupEvent,
        NotificationEvent, ControlMessage, WsError, WsStatus,
    };

    #[tokio::test]
    async fn test_websocket_request_creation() {
        let request = WebSocketFactory::basic_websocket_request();
        
        match request {
            WebSocketMessage::Request { id, action } => {
                assert!(!id.is_empty());
                match action {
                    ClientAction::Group(GroupAction::Join { group_id }) => {
                        assert_eq!(group_id, "1");
                    }
                    _ => panic!("Expected Join action"),
                }
            }
            _ => panic!("Expected Request message"),
        }
    }
    
    #[tokio::test]
    async fn test_join_group_request() {
        let request = WebSocketFactory::join_group_request("42");
        
        match request {
            WebSocketMessage::Request { id, action } => {
                assert!(!id.is_empty());
                match action {
                    ClientAction::Group(GroupAction::Join { group_id }) => {
                        assert_eq!(group_id, "42");
                    }
                    _ => panic!("Expected Join action"),
                }
            }
            _ => panic!("Expected Request message"),
        }
    }
    
    #[tokio::test]
    async fn test_leave_group_request() {
        let request = WebSocketFactory::leave_group_request("10");
        
        match request {
            WebSocketMessage::Request { id, action } => {
                assert!(!id.is_empty());
                match action {
                    ClientAction::Group(GroupAction::Leave { group_id }) => {
                        assert_eq!(group_id, "10");
                    }
                    _ => panic!("Expected Leave action"),
                }
            }
            _ => panic!("Expected Request message"),
        }
    }
    
    #[tokio::test]
    async fn test_new_message_request() {
        let request = WebSocketFactory::new_message_request("5", "Hello world!");
        
        match request {
            WebSocketMessage::Request { id, action } => {
                assert!(!id.is_empty());
                match action {
                    ClientAction::Group(GroupAction::NewMessage { group_id, content }) => {
                        assert_eq!(group_id, "5");
                        assert_eq!(content, "Hello world!");
                    }
                    _ => panic!("Expected NewMessage action"),
                }
            }
            _ => panic!("Expected Request message"),
        }
    }
    
    #[tokio::test]
    async fn test_websocket_success_response() {
        let response = WebSocketFactory::websocket_response("req123", true);
        
        match response {
            WebSocketMessage::Response { id, success, data, error } => {
                assert_eq!(id, "req123");
                assert!(success);
                assert!(data.is_some());
                assert!(error.is_none());
            }
            _ => panic!("Expected Response message"),
        }
    }
    
    #[tokio::test]
    async fn test_websocket_error_response() {
        let response = WebSocketFactory::websocket_response("req456", false);
        
        match response {
            WebSocketMessage::Response { id, success, data, error } => {
                assert_eq!(id, "req456");
                assert!(!success);
                assert!(data.is_none());
                assert!(error.is_some());
                
                let ws_error = error.unwrap();
                assert_eq!(ws_error.code, "test_error");
                assert_eq!(ws_error.message, "Test error message");
            }
            _ => panic!("Expected Response message"),
        }
    }
    
    #[tokio::test]
    async fn test_server_event_creation() {
        let event = WebSocketFactory::server_event();
        
        match event {
            WebSocketMessage::Event { event } => {
                match event {
                    ServerEvent::Groups(GroupEvent::MemberJoined { group_id, user_id, username }) => {
                        assert_eq!(group_id, "1");
                        assert_eq!(user_id, "1");
                        assert_eq!(username, "testuser");
                    }
                    _ => panic!("Expected MemberJoined event"),
                }
            }
            _ => panic!("Expected Event message"),
        }
    }
    
    #[tokio::test]
    async fn test_notification_event_creation() {
        let event = WebSocketFactory::notification_event();
        
        match event {
            WebSocketMessage::Event { event } => {
                match event {
                    ServerEvent::Notifications(NotificationEvent::NewMessage { 
                        group_id, message_id, content, sender_username 
                    }) => {
                        assert_eq!(group_id, "1");
                        assert_eq!(message_id, "1");
                        assert_eq!(content, "Test notification message");
                        assert_eq!(sender_username, "testuser");
                    }
                    _ => panic!("Expected NewMessage notification"),
                }
            }
            _ => panic!("Expected Event message"),
        }
    }
    
    #[tokio::test]
    async fn test_control_messages() {
        let connect = WebSocketFactory::connect_control();
        let disconnect = WebSocketFactory::disconnect_control();
        let ping = WebSocketFactory::ping_control();
        let pong = WebSocketFactory::pong_control();
        let ack = WebSocketFactory::ack_control("msg123");
        
        match connect {
            WebSocketMessage::Control { control } => {
                assert!(matches!(control, ControlMessage::Connect));
            }
            _ => panic!("Expected Control message"),
        }
        
        match disconnect {
            WebSocketMessage::Control { control } => {
                assert!(matches!(control, ControlMessage::Disconnect));
            }
            _ => panic!("Expected Control message"),
        }
        
        match ping {
            WebSocketMessage::Control { control } => {
                assert!(matches!(control, ControlMessage::Ping));
            }
            _ => panic!("Expected Control message"),
        }
        
        match pong {
            WebSocketMessage::Control { control } => {
                assert!(matches!(control, ControlMessage::Pong));
            }
            _ => panic!("Expected Control message"),
        }
        
        match ack {
            WebSocketMessage::Control { control } => {
                match control {
                    ControlMessage::Ack { message_id } => {
                        assert_eq!(message_id, "msg123");
                    }
                    _ => panic!("Expected Ack control"),
                }
            }
            _ => panic!("Expected Control message"),
        }
    }
    
    #[tokio::test]
    async fn test_ws_error_creation() {
        let mock_error = WebSocketFactory::mock_ws_error();
        assert_eq!(mock_error.code, "mock_error");
        assert_eq!(mock_error.message, "Mock error for testing");
        
        let conn_error = WebSocketFactory::connection_error();
        assert_eq!(conn_error.code, "connection_error");
        assert!(conn_error.message.contains("connection"));
        
        let auth_error = WebSocketFactory::auth_error();
        assert_eq!(auth_error.code, "auth_error");
        assert!(auth_error.message.contains("Authentication"));
    }
    
    #[tokio::test]
    async fn test_error_control_message() {
        let error = WebSocketFactory::mock_ws_error();
        let error_control = WebSocketFactory::error_control(error);
        
        match error_control {
            WebSocketMessage::Control { control } => {
                match control {
                    ControlMessage::Error(ws_error) => {
                        assert_eq!(ws_error.code, "mock_error");
                        assert_eq!(ws_error.message, "Mock error for testing");
                    }
                    _ => panic!("Expected Error control"),
                }
            }
            _ => panic!("Expected Control message"),
        }
    }
    
    #[tokio::test]
    async fn test_all_ws_status_variants() {
        let statuses = WebSocketFactory::all_ws_status();
        
        assert_eq!(statuses.len(), 4);
        assert!(statuses.contains(&WsStatus::Connecting));
        assert!(statuses.contains(&WsStatus::Open));
        assert!(statuses.contains(&WsStatus::Closed));
        assert!(statuses.contains(&WsStatus::Error));
    }
    
    #[tokio::test]
    async fn test_unique_request_ids() {
        let request1 = WebSocketFactory::join_group_request("1");
        let request2 = WebSocketFactory::join_group_request("1");
        
        let id1 = match request1 {
            WebSocketMessage::Request { id, .. } => id,
            _ => panic!("Expected Request"),
        };
        
        let id2 = match request2 {
            WebSocketMessage::Request { id, .. } => id,
            _ => panic!("Expected Request"),
        };
        
        assert_ne!(id1, id2, "Request IDs should be unique");
    }
}
