// WebSocket-related test data factory

use super::base_factory::BaseFactory;
use chrono::{DateTime, Utc};
use ruggine_client_ui::types::message_ws::{
    WebSocketMessage, ClientAction, GroupAction, ServerEvent, GroupEvent,
    NotificationEvent, ControlMessage, WsError, WsStatus
};

pub struct WebSocketFactory;

impl WebSocketFactory {
    /// Create mock group left event
    pub fn mock_ws_group_left_event(user_id: i32) -> WebSocketMessage {
        WebSocketMessage::Event {
            event: ServerEvent::Groups(GroupEvent::Left { user_id }),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Create mock new message event
    pub fn mock_ws_new_message_event(message_id: i32, group_id: i32, sender_id: i32, sender_username: &str, content: &str) -> WebSocketMessage {
        WebSocketMessage::Event {
            event: ServerEvent::Groups(GroupEvent::NewMessage {
                message_id,
                group_id,
                sender_id,
                sender_username: sender_username.to_string(),
                content: content.to_string(),
                sent_at: chrono::Utc::now(),
            }),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Create mock info notification
    pub fn mock_ws_info_notification(message: &str) -> WebSocketMessage {
        WebSocketMessage::Event {
            event: ServerEvent::Notifications(NotificationEvent::Info { 
                message: message.to_string() 
            }),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Create mock success notification
    pub fn mock_ws_success_notification(message: &str) -> WebSocketMessage {
        WebSocketMessage::Event {
            event: ServerEvent::Notifications(NotificationEvent::Success { 
                message: message.to_string() 
            }),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Create mock warning notification
    pub fn mock_ws_warning_notification(message: &str) -> WebSocketMessage {
        WebSocketMessage::Event {
            event: ServerEvent::Notifications(NotificationEvent::Warning { 
                message: message.to_string() 
            }),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Create mock error notification
    pub fn mock_ws_error_notification(message: &str) -> WebSocketMessage {
        WebSocketMessage::Event {
            event: ServerEvent::Notifications(NotificationEvent::Error { 
                message: message.to_string() 
            }),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Create mock connect control message
    pub fn mock_ws_connect_control() -> WebSocketMessage {
        WebSocketMessage::Control(ControlMessage::Connect)
    }
    
    /// Create mock disconnect control message
    pub fn mock_ws_disconnect_control() -> WebSocketMessage {
        WebSocketMessage::Control(ControlMessage::Disconnect)
    }
    
    /// Create mock ping control message
    pub fn mock_ws_ping_control() -> WebSocketMessage {
        WebSocketMessage::Control(ControlMessage::Ping)
    }
    
    /// Create mock pong control message
    pub fn mock_ws_pong_control() -> WebSocketMessage {
        WebSocketMessage::Control(ControlMessage::Pong)
    }
    
    /// Create mock ack control message
    pub fn mock_ws_ack_control(message_id: Option<&str>) -> WebSocketMessage {
        WebSocketMessage::Control(ControlMessage::Ack { 
            message_id: message_id.map(|s| s.to_string()) 
        })
    }
    
    /// Create mock error control message
    pub fn mock_ws_error_control(code: u16, message: &str, message_id: Option<&str>) -> WebSocketMessage {
        WebSocketMessage::Control(ControlMessage::Error {
            code,
            message: message.to_string(),
            message_id: message_id.map(|s| s.to_string()),
        })
    }
    
    /// Create mock validation error control message
    pub fn mock_ws_validation_error_control(code: u16, message: &str, message_id: Option<&str>) -> WebSocketMessage {
        WebSocketMessage::Control(ControlMessage::ValidationError {
            code,
            message: message.to_string(),
            message_id: message_id.map(|s| s.to_string()),
        })
    }
    
    /// Create mock WebSocket error
    pub fn mock_ws_error(code: u16, message: &str, message_id: Option<&str>) -> WsError {
        WsError {
            code,
            message: message.to_string(),
            message_id: message_id.map(|s| s.to_string()),
        }
    }
    
    /// Create mock WebSocket status progression
    pub fn mock_ws_status_progression() -> Vec<WsStatus> {
        vec![
            WsStatus::Connecting,
            WsStatus::Open,
            WsStatus::Closed,
        ]
    }
    
    /// Create mock error status progression
    pub fn mock_ws_error_status_progression() -> Vec<WsStatus> {
        vec![
            WsStatus::Connecting,
            WsStatus::Error("Connection failed".to_string()),
            WsStatus::Closed,
        ]
    }
    
    /// Create mock WebSocket message flow
    pub fn mock_ws_message_flow(group_id: i32, user_id: i32) -> Vec<WebSocketMessage> {
        vec![
            Self::mock_ws_join_request(),
            Self::mock_ws_success_response("test-123"),
            Self::mock_ws_group_joined_event(user_id),
            WebSocketMessage::Request {
                request_id: "msg-456".to_string(),
                action: ClientAction::Groups(GroupAction::NewMessage {
                    group_id,
                    content: "Hello everyone!".to_string(),
                }),
            },
            Self::mock_ws_success_response("msg-456"),
            Self::mock_ws_new_message_event(1, group_id, user_id, "testuser", "Hello everyone!"),
            WebSocketMessage::Request {
                request_id: "leave-789".to_string(),
                action: ClientAction::Groups(GroupAction::Leave {}),
            },
            Self::mock_ws_success_response("leave-789"),
            Self::mock_ws_group_left_event(user_id),
        ]
    }
    
    /// Create mock join request
    pub fn mock_ws_join_request() -> WebSocketMessage {
        WebSocketMessage::Request {
            request_id: BaseFactory::generate_uuid(),
            action: ClientAction::Groups(GroupAction::Join {}),
        }
    }
    
    /// Create mock success response
    pub fn mock_ws_success_response(request_id: &str) -> WebSocketMessage {
        use serde_json::json;
        WebSocketMessage::Response {
            request_id: request_id.to_string(),
            ok: true,
            data: Some(json!({"status": "success"})),
            error: None,
        }
    }
    
    /// Create mock group joined event
    pub fn mock_ws_group_joined_event(user_id: i32) -> WebSocketMessage {
        WebSocketMessage::Event {
            event: ServerEvent::Groups(GroupEvent::Joined { user_id }),
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Create mock error response
    pub fn mock_ws_error_response(request_id: &str, error_message: &str) -> WebSocketMessage {
        WebSocketMessage::Response {
            request_id: request_id.to_string(),
            ok: false,
            data: None,
            error: Some(WsError {
                code: 400,
                message: error_message.to_string(),
                message_id: Some(request_id.to_string()),
            }),
        }
    }
    /// Create basic WebSocket request for testing
    pub fn basic_websocket_request() -> WebSocketMessage {
        WebSocketMessage::Request {
            request_id: BaseFactory::get_unique_token(),
            action: ClientAction::Groups(GroupAction::Join {}),
        }
    }
    
    /// Create join group request
    pub fn join_group_request() -> WebSocketMessage {
        WebSocketMessage::Request {
            request_id: BaseFactory::get_unique_token(),
            action: ClientAction::Groups(GroupAction::Join {}),
        }
    }
    
    /// Create leave group request
    pub fn leave_group_request() -> WebSocketMessage {
        WebSocketMessage::Request {
            request_id: BaseFactory::get_unique_token(),
            action: ClientAction::Groups(GroupAction::Leave {}),
        }
    }
    
    /// Create new message request
    pub fn new_message_request(group_id: i32, content: &str) -> WebSocketMessage {
        WebSocketMessage::Request {
            request_id: BaseFactory::get_unique_token(),
            action: ClientAction::Groups(GroupAction::NewMessage {
                group_id,
                content: content.to_string(),
            }),
        }
    }
    
    /// Create test action request
    pub fn test_action_request(message: &str) -> WebSocketMessage {
        WebSocketMessage::Request {
            request_id: BaseFactory::get_unique_token(),
            action: ClientAction::Test {
                message: message.to_string(),
            },
        }
    }
    
    /// Create WebSocket response
    pub fn websocket_response(request_id: &str, ok: bool) -> WebSocketMessage {
        WebSocketMessage::Response {
            request_id: request_id.to_string(),
            ok,
            data: None,
            error: None,
        }
    }
    
    /// Create WebSocket error response
    pub fn websocket_error_response(request_id: &str, error: WsError) -> WebSocketMessage {
        WebSocketMessage::Response {
            request_id: request_id.to_string(),
            ok: false,
            data: None,
            error: Some(error),
        }
    }
    
    /// Create server event
    pub fn server_event(event: ServerEvent) -> WebSocketMessage {
        WebSocketMessage::Event {
            event,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// Create group joined event
    pub fn group_joined_event(user_id: i32) -> WebSocketMessage {
        Self::server_event(ServerEvent::Groups(GroupEvent::Joined { user_id }))
    }
    
    /// Create group left event
    pub fn group_left_event(user_id: i32) -> WebSocketMessage {
        Self::server_event(ServerEvent::Groups(GroupEvent::Left { user_id }))
    }
    
    /// Create new message event
    pub fn new_message_event(
        message_id: i32,
        group_id: i32,
        sender_id: i32,
        sender_username: &str,
        content: &str,
    ) -> WebSocketMessage {
        Self::server_event(ServerEvent::Groups(GroupEvent::NewMessage {
            message_id,
            group_id,
            sender_id,
            sender_username: sender_username.to_string(),
            content: content.to_string(),
            sent_at: chrono::Utc::now(),
        }))
    }
    
    /// Create notification event
    pub fn notification_info_event(message: &str) -> WebSocketMessage {
        Self::server_event(ServerEvent::Notifications(NotificationEvent::Info {
            message: message.to_string(),
        }))
    }
    
    /// Create notification success event
    pub fn notification_success_event(message: &str) -> WebSocketMessage {
        Self::server_event(ServerEvent::Notifications(NotificationEvent::Success {
            message: message.to_string(),
        }))
    }
    
    /// Create notification warning event
    pub fn notification_warning_event(message: &str) -> WebSocketMessage {
        Self::server_event(ServerEvent::Notifications(NotificationEvent::Warning {
            message: message.to_string(),
        }))
    }
    
    /// Create notification error event
    pub fn notification_error_event(message: &str) -> WebSocketMessage {
        Self::server_event(ServerEvent::Notifications(NotificationEvent::Error {
            message: message.to_string(),
        }))
    }
    
    /// Create control message
    pub fn control_message(control: ControlMessage) -> WebSocketMessage {
        WebSocketMessage::Control(control)
    }
    
    /// Create connect control message
    pub fn connect_control() -> WebSocketMessage {
        Self::control_message(ControlMessage::Connect)
    }
    
    /// Create disconnect control message
    pub fn disconnect_control() -> WebSocketMessage {
        Self::control_message(ControlMessage::Disconnect)
    }
    
    /// Create ping control message
    pub fn ping_control() -> WebSocketMessage {
        Self::control_message(ControlMessage::Ping)
    }
    
    /// Create pong control message
    pub fn pong_control() -> WebSocketMessage {
        Self::control_message(ControlMessage::Pong)
    }
    
    /// Create ack control message
    pub fn ack_control(message_id: Option<String>) -> WebSocketMessage {
        Self::control_message(ControlMessage::Ack { message_id })
    }
    
    /// Create error control message
    pub fn error_control(code: u16, message: &str, message_id: Option<String>) -> WebSocketMessage {
        Self::control_message(ControlMessage::Error {
            code,
            message: message.to_string(),
            message_id,
        })
    }
    
    /// Create validation error control message
    pub fn validation_error_control(code: u16, message: &str, message_id: Option<String>) -> WebSocketMessage {
        Self::control_message(ControlMessage::ValidationError {
            code,
            message: message.to_string(),
            message_id,
        })
    }
    
    /// Create WsError for testing
    pub fn ws_error(code: u16, message: &str, message_id: Option<String>) -> WsError {
        WsError {
            code,
            message: message.to_string(),
            message_id,
        }
    }
    
    /// Create WebSocket status variants for testing
    pub fn ws_status_variants() -> Vec<WsStatus> {
        vec![
            WsStatus::Connecting,
            WsStatus::Open,
            WsStatus::Closed,
            WsStatus::Error("Test error".to_string()),
        ]
    }
    
    /// Create request ID from WebSocketMessage
    pub fn extract_request_id(message: &WebSocketMessage) -> Option<String> {
        match message {
            WebSocketMessage::Request { request_id, .. } => Some(request_id.clone()),
            WebSocketMessage::Response { request_id, .. } => Some(request_id.clone()),
            _ => None,
        }
    }
    
    /// Mock WebSocket leave request
    pub fn mock_ws_leave_request() -> WebSocketMessage {
        WebSocketMessage::Request {
            request_id: BaseFactory::generate_uuid(),
            action: ClientAction::Groups(GroupAction::Leave {}),
        }
    }
    
    /// Mock WebSocket new message request
    pub fn mock_ws_new_message_request(group_id: i32, content: &str) -> WebSocketMessage {
        WebSocketMessage::Request {
            request_id: BaseFactory::generate_uuid(),
            action: ClientAction::Groups(GroupAction::NewMessage {
                group_id,
                content: content.to_string(),
            }),
        }
    }
    
    /// Mock WebSocket test request
    pub fn mock_ws_test_request(message: &str) -> WebSocketMessage {
        WebSocketMessage::Request {
            request_id: BaseFactory::generate_uuid(),
            action: ClientAction::Test {
                message: message.to_string(),
            },
        }
    }
}
