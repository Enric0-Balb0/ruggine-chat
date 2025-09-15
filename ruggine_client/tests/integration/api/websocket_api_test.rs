// Integration tests for WebSocket API

#[cfg(test)]
mod websocket_api_integration_tests {
    use crate::common::*;
    use ruggine_client_ui::types::message_ws::{
    WebSocketMessage, ClientAction, GroupAction, ServerEvent, GroupEvent,
    NotificationEvent, ControlMessage, WsStatus,
    };

    #[tokio::test]
    async fn test_websocket_request_creation() {
        let request = WebSocketFactory::basic_websocket_request();

        match request {
            WebSocketMessage::Request { request_id, action } => {
                assert!(!request_id.is_empty());
                match action {
                    ClientAction::Groups(GroupAction::Join {}) => {}
                    _ => panic!("Expected Join action"),
                }
            }
            _ => panic!("Expected Request message"),
        }
    }

    #[tokio::test]
    async fn test_join_leave_and_new_message_requests() {
        let join = WebSocketFactory::join_group_request();
        let leave = WebSocketFactory::leave_group_request();
        let new_msg = WebSocketFactory::new_message_request(5, "Hello world!");

        // join / leave are just Request with Join/Leave actions
        match join {
            WebSocketMessage::Request { request_id, action } => {
                assert!(!request_id.is_empty());
                matches!(action, ClientAction::Groups(GroupAction::Join {}));
            }
            _ => panic!("Expected Request"),
        }

        match leave {
            WebSocketMessage::Request { request_id, action } => {
                assert!(!request_id.is_empty());
                matches!(action, ClientAction::Groups(GroupAction::Leave {}));
            }
            _ => panic!("Expected Request"),
        }

        match new_msg {
            WebSocketMessage::Request { request_id, action } => {
                assert!(!request_id.is_empty());
                match action {
                    ClientAction::Groups(GroupAction::NewMessage { group_id, content }) => {
                        assert_eq!(group_id, 5);
                        assert_eq!(content, "Hello world!");
                    }
                    _ => panic!("Expected NewMessage action"),
                }
            }
            _ => panic!("Expected Request message"),
        }
    }

    #[tokio::test]
    async fn test_websocket_responses_and_errors() {
        let ok = WebSocketFactory::mock_ws_success_response("req123");
        let err = WebSocketFactory::websocket_error_response(
            "req456",
            WebSocketFactory::ws_error(400, "Test error", Some("req456".to_string())),
        );

        match ok {
            WebSocketMessage::Response { request_id, ok, data, error } => {
                assert_eq!(request_id, "req123");
                assert!(ok);
                assert!(data.is_some(), "Success responses should include data");
                assert!(error.is_none());
            }
            _ => panic!("Expected Response message"),
        }

        match err {
            WebSocketMessage::Response { request_id, ok, data, error } => {
                assert_eq!(request_id, "req456");
                assert!(!ok);
                assert!(data.is_none());
                assert!(error.is_some());
                let ws_err = error.unwrap();
                assert_eq!(ws_err.code, 400);
                assert!(ws_err.message.contains("Test error"));
            }
            _ => panic!("Expected Response message"),
        }
    }

    #[tokio::test]
    async fn test_server_events_and_notifications() {
        let joined = WebSocketFactory::group_joined_event(1);
        let new_msg = WebSocketFactory::mock_ws_new_message_event(1, 1, 1, "testuser", "hi");
        let info = WebSocketFactory::notification_info_event("info");

        match joined {
            WebSocketMessage::Event { event, timestamp: _ } => {
                match event {
                    ServerEvent::Groups(GroupEvent::Joined { user_id }) => {
                        assert_eq!(user_id, 1);
                    }
                    _ => panic!("Expected Joined event"),
                }
            }
            _ => panic!("Expected Event message"),
        }

        match new_msg {
            WebSocketMessage::Event { event, timestamp: _ } => {
                match event {
                    ServerEvent::Groups(GroupEvent::NewMessage { message_id, group_id, sender_id, sender_username, content, .. }) => {
                        assert_eq!(message_id, 1);
                        assert_eq!(group_id, 1);
                        assert_eq!(sender_username, "testuser");
                    }
                    _ => panic!("Expected NewMessage event"),
                }
            }
            _ => panic!("Expected Event message"),
        }

        match info {
            WebSocketMessage::Event { event, timestamp: _ } => {
                match event {
                    ServerEvent::Notifications(NotificationEvent::Info { message }) => {
                        assert!(message.contains("info"));
                    }
                    _ => panic!("Expected Info notification"),
                }
            }
            _ => panic!("Expected Event message"),
        }
    }

    #[tokio::test]
    async fn test_control_messages_and_ack() {
        let connect = WebSocketFactory::connect_control();
        let disconnect = WebSocketFactory::disconnect_control();
        let ping = WebSocketFactory::ping_control();
        let pong = WebSocketFactory::pong_control();
        let ack = WebSocketFactory::ack_control(Some("msg123".to_string()));

        match connect {
            WebSocketMessage::Control(control) => assert_eq!(control, ControlMessage::Connect),
            _ => panic!("Expected Control message"),
        }

        match disconnect {
            WebSocketMessage::Control(control) => assert_eq!(control, ControlMessage::Disconnect),
            _ => panic!("Expected Control message"),
        }

        match ping {
            WebSocketMessage::Control(control) => assert_eq!(control, ControlMessage::Ping),
            _ => panic!("Expected Control message"),
        }

        match pong {
            WebSocketMessage::Control(control) => assert_eq!(control, ControlMessage::Pong),
            _ => panic!("Expected Control message"),
        }

        match ack {
            WebSocketMessage::Control(control) => match control {
                ControlMessage::Ack { message_id } => assert_eq!(message_id, Some("msg123".to_string())),
                _ => panic!("Expected Ack control"),
            },
            _ => panic!("Expected Control message"),
        }
    }

    #[tokio::test]
    async fn test_ws_error_helpers() {
    let mock_error = WebSocketFactory::mock_ws_error(400, "Mock error for testing", Some("id"));
        assert_eq!(mock_error.code, 400);
        assert!(mock_error.message.contains("Mock error"));

    let ws_err = WebSocketFactory::ws_error(401, "Auth error", Some("id".to_string()));
        assert_eq!(ws_err.code, 401);
    }

    #[tokio::test]
    async fn test_status_variants_and_unique_ids() {
        let statuses = WebSocketFactory::ws_status_variants();
        assert_eq!(statuses.len(), 4);
        assert!(statuses.contains(&WsStatus::Connecting));
        assert!(statuses.contains(&WsStatus::Open));
        assert!(statuses.contains(&WsStatus::Closed));
        assert!(statuses.contains(&WsStatus::Error("Test error".to_string())));

        let r1 = WebSocketFactory::join_group_request();
        let r2 = WebSocketFactory::join_group_request();
        let id1 = WebSocketFactory::extract_request_id(&r1).unwrap();
        let id2 = WebSocketFactory::extract_request_id(&r2).unwrap();
        assert_ne!(id1, id2);
    }
}

