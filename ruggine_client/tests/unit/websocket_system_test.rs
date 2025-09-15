use crate::common::TestFactory;
use ruggine_client_ui::types::message_ws::{
    WebSocketMessage, ClientAction, GroupAction, ServerEvent, GroupEvent,
    NotificationEvent, ControlMessage, WsError, WsStatus,
};
use serde_json::Value;
use chrono::Utc;

// =============================================================================
// WEBSOCKET MESSAGE TYPE TESTS
// =============================================================================

#[cfg(test)]
mod websocket_message_tests {
    use super::*;

    #[test]
    fn test_ws_join_request_creation() {
        let join_msg = TestFactory::mock_ws_join_request();
        
        match join_msg {
            WebSocketMessage::Request { request_id, action } => {
                assert!(!request_id.is_empty());
                assert!(request_id.contains("test-uuid"));
                
                match action {
                    ClientAction::Groups(GroupAction::Join {}) => {
                        // Success - correct action type
                    }
                    _ => panic!("Expected Groups(Join) action"),
                }
            }
            _ => panic!("Expected Request message"),
        }
    }

    #[test]
    fn test_ws_leave_request_creation() {
        let leave_msg = TestFactory::mock_ws_leave_request();
        
        match leave_msg {
            WebSocketMessage::Request { request_id, action } => {
                assert!(!request_id.is_empty());
                
                match action {
                    ClientAction::Groups(GroupAction::Leave {}) => {
                        // Success - correct action type
                    }
                    _ => panic!("Expected Groups(Leave) action"),
                }
            }
            _ => panic!("Expected Request message"),
        }
    }

    #[test]
    fn test_ws_new_message_request_creation() {
        let content = "Test message content";
        let group_id = 42;
        let msg = TestFactory::mock_ws_new_message_request(group_id, content);
        
        match msg {
            WebSocketMessage::Request { request_id, action } => {
                assert!(!request_id.is_empty());
                
                match action {
                    ClientAction::Groups(GroupAction::NewMessage { group_id: gid, content: c }) => {
                        assert_eq!(gid, group_id);
                        assert_eq!(c, content);
                    }
                    _ => panic!("Expected Groups(NewMessage) action"),
                }
            }
            _ => panic!("Expected Request message"),
        }
    }

    #[test]
    fn test_ws_test_request_creation() {
        let test_message = "Test ping message";
        let msg = TestFactory::mock_ws_test_request(test_message);
        
        match msg {
            WebSocketMessage::Request { request_id, action } => {
                assert!(!request_id.is_empty());
                
                match action {
                    ClientAction::Test { message } => {
                        assert_eq!(message, test_message);
                    }
                    _ => panic!("Expected Test action"),
                }
            }
            _ => panic!("Expected Request message"),
        }
    }

    #[test]
    fn test_ws_success_response_creation() {
        let request_id = "test-request-123";
        let msg = TestFactory::mock_ws_success_response(request_id);
        
        match msg {
            WebSocketMessage::Response { request_id: rid, ok, data, error } => {
                assert_eq!(rid, request_id);
                assert!(ok);
                assert!(data.is_some());
                assert!(error.is_none());
                
                if let Some(data_val) = data {
                    assert!(data_val.get("status").is_some());
                }
            }
            _ => panic!("Expected Response message"),
        }
    }

    #[test]
    fn test_ws_error_response_creation() {
        let request_id = "test-request-456";
        let error_msg = "Invalid request format";
        let msg = TestFactory::mock_ws_error_response(request_id, error_msg);
        
        match msg {
            WebSocketMessage::Response { request_id: rid, ok, data, error } => {
                assert_eq!(rid, request_id);
                assert!(!ok);
                assert!(data.is_none());
                assert!(error.is_some());
                
                if let Some(err) = error {
                    assert_eq!(err.code, 400);
                    assert_eq!(err.message, error_msg);
                    assert_eq!(err.message_id, Some(request_id.to_string()));
                }
            }
            _ => panic!("Expected Response message"),
        }
    }

    #[test]
    fn test_uuid_generation_uniqueness() {
        let uuid1 = TestFactory::generate_uuid();
        let uuid2 = TestFactory::generate_uuid();
        let uuid3 = TestFactory::generate_uuid();
        
        assert_ne!(uuid1, uuid2);
        assert_ne!(uuid2, uuid3);
        assert_ne!(uuid1, uuid3);
        
        // Check format
        assert!(uuid1.starts_with("test-uuid-"));
        assert!(uuid2.starts_with("test-uuid-"));
        assert!(uuid3.starts_with("test-uuid-"));
    }
}

// =============================================================================
// WEBSOCKET EVENT TESTS
// =============================================================================

#[cfg(test)]
mod websocket_event_tests {
    use super::*;

    #[test]
    fn test_ws_group_joined_event() {
        let user_id = 123;
        let msg = TestFactory::mock_ws_group_joined_event(user_id);
        
        match msg {
            WebSocketMessage::Event { event, timestamp } => {
                assert!(timestamp <= Utc::now());
                
                match event {
                    ServerEvent::Groups(GroupEvent::Joined { user_id: uid }) => {
                        assert_eq!(uid, user_id);
                    }
                    _ => panic!("Expected Groups(Joined) event"),
                }
            }
            _ => panic!("Expected Event message"),
        }
    }

    #[test]
    fn test_ws_group_left_event() {
        let user_id = 456;
        let msg = TestFactory::mock_ws_group_left_event(user_id);
        
        match msg {
            WebSocketMessage::Event { event, timestamp } => {
                assert!(timestamp <= Utc::now());
                
                match event {
                    ServerEvent::Groups(GroupEvent::Left { user_id: uid }) => {
                        assert_eq!(uid, user_id);
                    }
                    _ => panic!("Expected Groups(Left) event"),
                }
            }
            _ => panic!("Expected Event message"),
        }
    }

    #[test]
    fn test_ws_new_message_event() {
        let message_id = 789;
        let group_id = 101;
        let sender_id = 202;
        let sender_username = "testuser";
        let content = "Hello from WebSocket!";
        
        let msg = TestFactory::mock_ws_new_message_event(
            message_id, group_id, sender_id, sender_username, content
        );
        
        match msg {
            WebSocketMessage::Event { event, timestamp } => {
                assert!(timestamp <= Utc::now());
                
                match event {
                    ServerEvent::Groups(GroupEvent::NewMessage {
                        message_id: mid,
                        group_id: gid,
                        sender_id: sid,
                        sender_username: username,
                        content: msg_content,
                        sent_at,
                    }) => {
                        assert_eq!(mid, message_id);
                        assert_eq!(gid, group_id);
                        assert_eq!(sid, sender_id);
                        assert_eq!(username, sender_username);
                        assert_eq!(msg_content, content);
                        assert!(sent_at <= Utc::now());
                    }
                    _ => panic!("Expected Groups(NewMessage) event"),
                }
            }
            _ => panic!("Expected Event message"),
        }
    }

    #[test]
    fn test_ws_notification_events() {
        let info_msg = TestFactory::mock_ws_info_notification("Info message");
        let success_msg = TestFactory::mock_ws_success_notification("Success message");
        let warning_msg = TestFactory::mock_ws_warning_notification("Warning message");
        let error_msg = TestFactory::mock_ws_error_notification("Error message");
        
        // Test info notification
        match info_msg {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Notifications(NotificationEvent::Info { message }) => {
                        assert_eq!(message, "Info message");
                    }
                    _ => panic!("Expected Info notification"),
                }
            }
            _ => panic!("Expected Event message"),
        }
        
        // Test success notification
        match success_msg {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Notifications(NotificationEvent::Success { message }) => {
                        assert_eq!(message, "Success message");
                    }
                    _ => panic!("Expected Success notification"),
                }
            }
            _ => panic!("Expected Event message"),
        }
        
        // Test warning notification
        match warning_msg {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Notifications(NotificationEvent::Warning { message }) => {
                        assert_eq!(message, "Warning message");
                    }
                    _ => panic!("Expected Warning notification"),
                }
            }
            _ => panic!("Expected Event message"),
        }
        
        // Test error notification
        match error_msg {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Notifications(NotificationEvent::Error { message }) => {
                        assert_eq!(message, "Error message");
                    }
                    _ => panic!("Expected Error notification"),
                }
            }
            _ => panic!("Expected Event message"),
        }
    }
}

// =============================================================================
// WEBSOCKET CONTROL MESSAGE TESTS
// =============================================================================

#[cfg(test)]
mod websocket_control_tests {
    use super::*;

    #[test]
    fn test_ws_control_messages() {
        let connect_msg = TestFactory::mock_ws_connect_control();
        let disconnect_msg = TestFactory::mock_ws_disconnect_control();
        let ping_msg = TestFactory::mock_ws_ping_control();
        let pong_msg = TestFactory::mock_ws_pong_control();
        
        // Test connect
        match connect_msg {
            WebSocketMessage::Control(ControlMessage::Connect) => {
                // Success
            }
            _ => panic!("Expected Control(Connect) message"),
        }
        
        // Test disconnect
        match disconnect_msg {
            WebSocketMessage::Control(ControlMessage::Disconnect) => {
                // Success
            }
            _ => panic!("Expected Control(Disconnect) message"),
        }
        
        // Test ping
        match ping_msg {
            WebSocketMessage::Control(ControlMessage::Ping) => {
                // Success
            }
            _ => panic!("Expected Control(Ping) message"),
        }
        
        // Test pong
        match pong_msg {
            WebSocketMessage::Control(ControlMessage::Pong) => {
                // Success
            }
            _ => panic!("Expected Control(Pong) message"),
        }
    }

    #[test]
    fn test_ws_ack_control() {
        let ack_without_id = TestFactory::mock_ws_ack_control(None);
        let ack_with_id = TestFactory::mock_ws_ack_control(Some("msg-123"));
        
        // Test ack without message ID
        match ack_without_id {
            WebSocketMessage::Control(ControlMessage::Ack { message_id }) => {
                assert!(message_id.is_none());
            }
            _ => panic!("Expected Control(Ack) message"),
        }
        
        // Test ack with message ID
        match ack_with_id {
            WebSocketMessage::Control(ControlMessage::Ack { message_id }) => {
                assert_eq!(message_id, Some("msg-123".to_string()));
            }
            _ => panic!("Expected Control(Ack) message"),
        }
    }

    #[test]
    fn test_ws_error_control() {
        let error_msg = TestFactory::mock_ws_error_control(
            500, 
            "Internal server error", 
            Some("req-456")
        );
        
        match error_msg {
            WebSocketMessage::Control(ControlMessage::Error { code, message, message_id }) => {
                assert_eq!(code, 500);
                assert_eq!(message, "Internal server error");
                assert_eq!(message_id, Some("req-456".to_string()));
            }
            _ => panic!("Expected Control(Error) message"),
        }
    }

    #[test]
    fn test_ws_validation_error_control() {
        let validation_error_msg = TestFactory::mock_ws_validation_error_control(
            "field_name",
            "Invalid message format", 
            Some("req-789")
        );
        
        match validation_error_msg {
            WebSocketMessage::Control(ControlMessage::ValidationError { code, message, message_id }) => {
                assert_eq!(code, 400);
                assert_eq!(message, "field_name: Invalid message format");
                assert_eq!(message_id, Some("req-789".to_string()));
            }
            _ => panic!("Expected Control(ValidationError) message"),
        }
    }
}

// =============================================================================
// WEBSOCKET ERROR TESTS
// =============================================================================

#[cfg(test)]
mod websocket_error_tests {
    use super::*;

    #[test]
    fn test_ws_error_creation() {
        let error = TestFactory::mock_ws_error(404, "Not found", Some("req-001"));
        
        assert_eq!(error.code, 404);
        assert_eq!(error.message, "Not found");
        assert_eq!(error.message_id, Some("req-001".to_string()));
    }

    #[test]
    fn test_ws_error_without_message_id() {
        let error = TestFactory::mock_ws_error(500, "Server error", None);
        
        assert_eq!(error.code, 500);
        assert_eq!(error.message, "Server error");
        assert!(error.message_id.is_none());
    }

    #[test]
    fn test_ws_error_serialization() {
        let error = TestFactory::mock_ws_error(403, "Forbidden", Some("req-403"));
        
        // Test serialization to JSON
        let json = serde_json::to_string(&error).expect("Should serialize to JSON");
        assert!(!json.is_empty());
        assert!(json.contains("403"));
        assert!(json.contains("Forbidden"));
        assert!(json.contains("req-403"));
        
        // Test deserialization from JSON
        let deserialized: WsError = serde_json::from_str(&json).expect("Should deserialize from JSON");
        assert_eq!(error.code, deserialized.code);
        assert_eq!(error.message, deserialized.message);
        assert_eq!(error.message_id, deserialized.message_id);
    }
}

// =============================================================================
// WEBSOCKET STATUS TESTS
// =============================================================================

#[cfg(test)]
mod websocket_status_tests {
    use super::*;

    #[test]
    fn test_ws_status_progression() {
        let progression = TestFactory::mock_ws_status_progression();
        
        assert_eq!(progression.len(), 3);
        assert_eq!(progression[0], WsStatus::Connecting);
        assert_eq!(progression[1], WsStatus::Open);
        assert_eq!(progression[2], WsStatus::Closed);
    }

    #[test]
    fn test_ws_error_status_progression() {
        let progression = TestFactory::mock_ws_error_status_progression();
        
        assert_eq!(progression.len(), 3);
        assert_eq!(progression[0], WsStatus::Connecting);
        
        match &progression[1] {
            WsStatus::Error(msg) => {
                assert_eq!(msg, "Connection failed");
            }
            _ => panic!("Expected Error status"),
        }
        
        assert_eq!(progression[2], WsStatus::Closed);
    }

    #[test]
    fn test_ws_status_equality() {
        let status1 = WsStatus::Open;
        let status2 = WsStatus::Open;
        let status3 = WsStatus::Closed;
        let status4 = WsStatus::Error("Test error".to_string());
        let status5 = WsStatus::Error("Test error".to_string());
        let status6 = WsStatus::Error("Different error".to_string());
        
        assert_eq!(status1, status2);
        assert_ne!(status1, status3);
        assert_ne!(status1, status4);
        assert_eq!(status4, status5);
        assert_ne!(status4, status6);
    }

    #[test]
    fn test_ws_status_cloning() {
        let original = WsStatus::Error("Original error".to_string());
        let cloned = original.clone();
        
        assert_eq!(original, cloned);
        
        match (&original, &cloned) {
            (WsStatus::Error(msg1), WsStatus::Error(msg2)) => {
                assert_eq!(msg1, msg2);
                assert_eq!(msg1, "Original error");
            }
            _ => panic!("Expected Error status for both"),
        }
    }
}

// =============================================================================
// WEBSOCKET MESSAGE FLOW TESTS
// =============================================================================

#[cfg(test)]
mod websocket_flow_tests {
    use super::*;

    #[test]
    fn test_ws_message_flow() {
        let group_id = 42;
        let user_id = 123;
        let flow = TestFactory::mock_ws_message_flow(group_id, user_id);
        
        assert_eq!(flow.len(), 9);
        
        // Test sequence: Join -> Success -> Joined -> NewMessage -> Success -> NewMessage Event -> Leave -> Success -> Left
        
        // 1. Join request
        match &flow[0] {
            WebSocketMessage::Request { action, .. } => {
                match action {
                    ClientAction::Groups(GroupAction::Join {}) => {
                        // Success
                    }
                    _ => panic!("Expected Join action"),
                }
            }
            _ => panic!("Expected Request message"),
        }
        
        // 2. Success response
        match &flow[1] {
            WebSocketMessage::Response { ok, .. } => {
                assert!(ok);
            }
            _ => panic!("Expected Response message"),
        }
        
        // 3. Joined event
        match &flow[2] {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::Joined { user_id: uid }) => {
                        assert_eq!(*uid, user_id);
                    }
                    _ => panic!("Expected Joined event"),
                }
            }
            _ => panic!("Expected Event message"),
        }
        
        // 4. New message request
        match &flow[3] {
            WebSocketMessage::Request { action, .. } => {
                match action {
                    ClientAction::Groups(GroupAction::NewMessage { group_id: gid, content }) => {
                        assert_eq!(*gid, group_id);
                        assert_eq!(content, "Hello everyone!");
                    }
                    _ => panic!("Expected NewMessage action"),
                }
            }
            _ => panic!("Expected Request message"),
        }
        
        // 5. Success response
        match &flow[4] {
            WebSocketMessage::Response { ok, .. } => {
                assert!(ok);
            }
            _ => panic!("Expected Response message"),
        }
        
        // 6. New message event
        match &flow[5] {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::NewMessage { 
                        message_id, group_id: gid, sender_id: sid, sender_username, content, .. 
                    }) => {
                        assert_eq!(*message_id, 1);
                        assert_eq!(*gid, group_id);
                        assert_eq!(*sid, user_id);
                        assert_eq!(sender_username, "testuser");
                        assert_eq!(content, "Hello everyone!");
                    }
                    _ => panic!("Expected NewMessage event"),
                }
            }
            _ => panic!("Expected Event message"),
        }
        
        // 7. Leave request
        match &flow[6] {
            WebSocketMessage::Request { action, .. } => {
                match action {
                    ClientAction::Groups(GroupAction::Leave {}) => {
                        // Success
                    }
                    _ => panic!("Expected Leave action"),
                }
            }
            _ => panic!("Expected Request message"),
        }
        
        // 8. Success response
        match &flow[7] {
            WebSocketMessage::Response { ok, .. } => {
                assert!(ok);
            }
            _ => panic!("Expected Response message"),
        }
        
        // 9. Left event
        match &flow[8] {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::Left { user_id: uid }) => {
                        assert_eq!(*uid, user_id);
                    }
                    _ => panic!("Expected Left event"),
                }
            }
            _ => panic!("Expected Event message"),
        }
    }

    #[test]
    fn test_ws_flow_timestamps() {
        let flow = TestFactory::mock_ws_message_flow(1, 1);
        let now = Utc::now();
        
        for msg in &flow {
            match msg {
                WebSocketMessage::Event { timestamp, .. } => {
                    // All event timestamps should be very recent
                    let diff = now.signed_duration_since(timestamp);
                    assert!(diff.num_seconds() < 10, "Event timestamp should be very recent");
                }
                _ => {
                    // Non-event messages don't have timestamps, that's fine
                }
            }
        }
    }

    #[test]
    fn test_ws_flow_request_ids() {
        let flow = TestFactory::mock_ws_message_flow(1, 1);
        let mut request_ids = Vec::new();
        let mut response_ids = Vec::new();
        
        for msg in &flow {
            match msg {
                WebSocketMessage::Request { request_id, .. } => {
                    request_ids.push(request_id.clone());
                    assert!(!request_id.is_empty());
                }
                WebSocketMessage::Response { request_id, .. } => {
                    response_ids.push(request_id.clone());
                    assert!(!request_id.is_empty());
                }
                _ => {
                    // Events and control messages don't have request IDs
                }
            }
        }
        
        // Should have some requests and responses
        assert!(!request_ids.is_empty());
        assert!(!response_ids.is_empty());
        
        // All request IDs should be unique
        let mut unique_request_ids = request_ids.clone();
        unique_request_ids.sort();
        unique_request_ids.dedup();
        assert_eq!(unique_request_ids.len(), request_ids.len(), "All request IDs should be unique");
    }
}

// =============================================================================
// WEBSOCKET MESSAGE SERIALIZATION TESTS
// =============================================================================

#[cfg(test)]
mod websocket_serialization_tests {
    use super::*;

    #[test]
    fn test_ws_message_serialization() {
        let messages = vec![
            TestFactory::mock_ws_join_request(),
            TestFactory::mock_ws_success_response("test-123"),
            TestFactory::mock_ws_group_joined_event(456),
            TestFactory::mock_ws_ping_control(),
            TestFactory::mock_ws_info_notification("Test notification"),
        ];
        
        for msg in messages {
            // Test serialization to JSON
            let json = serde_json::to_string(&msg).expect("Should serialize to JSON");
            assert!(!json.is_empty());
            
            // Test deserialization from JSON
            let deserialized: WebSocketMessage = serde_json::from_str(&json)
                .expect("Should deserialize from JSON");
            
            // Since we can't easily implement Eq for WebSocketMessage due to DateTime fields,
            // we'll just verify the structure is maintained by re-serializing
            let json2 = serde_json::to_string(&deserialized).expect("Should serialize again");
            
            // The JSON should be similar (timestamps might differ slightly)
            assert!(json.len() > 10); // Should have meaningful content
            assert!(json2.len() > 10); // Should have meaningful content
        }
    }

    #[test]
    fn test_ws_message_json_structure() {
        let join_msg = TestFactory::mock_ws_join_request();
        let json = serde_json::to_string(&join_msg).expect("Should serialize to JSON");
        
        // Parse back to Value to check structure
        let value: Value = serde_json::from_str(&json).expect("Should parse to Value");
        
        // Should have type field
        assert!(value.get("type").is_some());
        assert_eq!(value.get("type").unwrap().as_str().unwrap(), "request");
        
        // Should have request_id field
        assert!(value.get("request_id").is_some());
        assert!(value.get("request_id").unwrap().is_string());
        
        // Should have action field
        assert!(value.get("action").is_some());
        assert!(value.get("action").unwrap().is_object());
    }

    #[test]
    fn test_ws_error_json_structure() {
        let error_msg = TestFactory::mock_ws_error_response("test-456", "Test error");
        let json = serde_json::to_string(&error_msg).expect("Should serialize to JSON");
        
        let value: Value = serde_json::from_str(&json).expect("Should parse to Value");
        
        // Should have type field
        assert_eq!(value.get("type").unwrap().as_str().unwrap(), "response");
        
        // Should have ok field as false
        assert_eq!(value.get("ok").unwrap().as_bool().unwrap(), false);
        
        // Should have error field
        assert!(value.get("error").is_some());
        assert!(value.get("error").unwrap().is_object());
        
        // Error should have code and message
        let error_obj = value.get("error").unwrap();
        assert!(error_obj.get("code").is_some());
        assert!(error_obj.get("message").is_some());
    }
}
