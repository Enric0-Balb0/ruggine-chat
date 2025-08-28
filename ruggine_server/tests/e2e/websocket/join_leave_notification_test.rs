use axum::http::StatusCode;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;
use tokio::time::timeout;

use ruggine_server::websocket::group_message::{GroupAction, GroupEvent};
use ruggine_server::websocket::message::{ClientAction, ControlMessage, ServerEvent, WebSocketMessage};

use crate::common::{
    cleanup_group_chat, cleanup_test_users, cleanup_test_users_from_a_group_chat,
    create_login_and_get_token, create_test_group_chat_with_invitation_and_membership,
    create_test_users_for_a_group, connect_chat_websocket_with_auth,
    send_websocket_message_and_get_response, start_test_server,
};

#[cfg(test)]
mod group_websocket_join_leave_notification_e2e_tests {
    use super::*;

    /// Helper function to receive a websocket message with timeout
    async fn receive_websocket_message_with_timeout(
        ws_stream: &mut tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        timeout_duration: Duration,
    ) -> Result<Option<WebSocketMessage>, Box<dyn std::error::Error>> {
        match timeout(timeout_duration, ws_stream.next()).await {
            Ok(Some(msg)) => match msg? {
                Message::Text(text) => {
                    let parsed: WebSocketMessage = serde_json::from_str(&text)?;
                    Ok(Some(parsed))
                }
                _ => Ok(None),
            },
            Ok(None) => Ok(None),
            Err(_) => Ok(None), // Timeout
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_user_join_signals_connected_users() {
        // Arrange: Create two users in the same group
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_join_signal_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_join_signal_user2".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_join_signal_group", user1.id).await;
        
        // Add user2 to the same group to create a connection between users
        let _membership2 = crate::common::add_test_user_to_a_group(user2.id, &group).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user1 first and join the group
        let (mut ws1, _) = connect_chat_websocket_with_auth(addr, &token1)
            .await
            .expect("Failed to connect user1 to websocket");

        let join_id1 = Uuid::new_v4().to_string();
        let join_msg1 = WebSocketMessage::Request {
            request_id: join_id1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response1 = send_websocket_message_and_get_response(&mut ws1, join_msg1)
            .await
            .expect("User1 failed to join group");

        // Connect user2 to websocket but don't join yet
        let (mut ws2, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 to websocket");

        // Act: User2 joins the group - this should signal user1
        let join_id2 = Uuid::new_v4().to_string();
        let join_msg2 = WebSocketMessage::Request {
            request_id: join_id2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let response2 = send_websocket_message_and_get_response(&mut ws2, join_msg2)
            .await
            .expect("User2 failed to join group");

        // Assert: User2 should successfully join
        match response2 {
            WebSocketMessage::Response { ok, error, .. } => {
                assert!(ok, "User2 should successfully join group");
                assert!(error.is_none(), "Should not have error");
            }
            _ => panic!("Expected Response message for user2 join"),
        }

        // Assert: User1 should receive a notification about user2 joining
        let notification = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have received a message");

        match notification {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::Joined { user_id }) => {
                    assert_eq!(user_id, user2.id, "Should receive notification about user2 joining");
                }
                _ => panic!("Expected GroupEvent::Joined, got: {:?}", event),
            },
            _ => panic!("Expected Event message, got: {:?}", notification),
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id, user2.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_user_leave_signals_connected_users() {
        // Arrange: Create two users in the same group and both connected
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_leave_signal_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_leave_signal_user2".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_leave_signal_group", user1.id).await;
        
        // Add user2 to the same group
        let _membership2 = crate::common::add_test_user_to_a_group(user2.id, &group).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect both users and join the group
        let (mut ws1, _) = connect_chat_websocket_with_auth(addr, &token1)
            .await
            .expect("Failed to connect user1 to websocket");

        let join_id1 = Uuid::new_v4().to_string();
        let join_msg1 = WebSocketMessage::Request {
            request_id: join_id1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response1 = send_websocket_message_and_get_response(&mut ws1, join_msg1)
            .await
            .expect("User1 failed to join group");

        let (mut ws2, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 to websocket");

        let join_id2 = Uuid::new_v4().to_string();
        let join_msg2 = WebSocketMessage::Request {
            request_id: join_id2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2 = send_websocket_message_and_get_response(&mut ws2, join_msg2)
            .await
            .expect("User2 failed to join group");

        // Clear any pending join notifications
        let _ = receive_websocket_message_with_timeout(&mut ws1, Duration::from_millis(500)).await;

        // Act: User2 leaves the group - this should signal user1
        let leave_id2 = Uuid::new_v4().to_string();
        let leave_msg2 = WebSocketMessage::Request {
            request_id: leave_id2.clone(),
            action: ClientAction::Groups(GroupAction::Leave {}),
        };
        let response2 = send_websocket_message_and_get_response(&mut ws2, leave_msg2)
            .await
            .expect("User2 failed to send leave message");

        // Assert: User2 should successfully leave
        match response2 {
            WebSocketMessage::Response { ok, error, .. } => {
                assert!(ok, "User2 should successfully leave group");
                assert!(error.is_none(), "Should not have error");
            }
            _ => panic!("Expected Response message for user2 leave"),
        }

        // Assert: User1 should receive a notification about user2 leaving
        let notification = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have received a message");

        match notification {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::Left { user_id }) => {
                    assert_eq!(user_id, user2.id, "Should receive notification about user2 leaving");
                }
                _ => panic!("Expected GroupEvent::Left, got: {:?}", event),
            },
            _ => panic!("Expected Event message, got: {:?}", notification),
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id, user2.id], group.id).await; // user2 already left
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_multiple_users_join_notifications() {
        // Arrange: Create multiple users in the same group
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_multi_join_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_multi_join_user2".to_string()).await;
        let (user3, _password3, token3) =
            create_login_and_get_token("e2e_multi_join_user3".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_multi_join_group", user1.id).await;
        
        // Add other users to the same group
        let _membership2 = crate::common::add_test_user_to_a_group(user2.id, &group).await;
        let _membership3 = crate::common::add_test_user_to_a_group(user3.id, &group).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user1 first and join the group
        let (mut ws1, _) = connect_chat_websocket_with_auth(addr, &token1)
            .await
            .expect("Failed to connect user1 to websocket");

        let join_id1 = Uuid::new_v4().to_string();
        let join_msg1 = WebSocketMessage::Request {
            request_id: join_id1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response1 = send_websocket_message_and_get_response(&mut ws1, join_msg1)
            .await
            .expect("User1 failed to join group");

        // Connect user2 but don't join yet
        let (mut ws2, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 to websocket");

        // Act: User2 joins - user1 should be notified
        let join_id2 = Uuid::new_v4().to_string();
        let join_msg2 = WebSocketMessage::Request {
            request_id: join_id2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2 = send_websocket_message_and_get_response(&mut ws2, join_msg2)
            .await
            .expect("User2 failed to join group");

        // Assert: User1 receives notification about user2
        let notification1 = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have received a message");

        match notification1 {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::Joined { user_id }) => {
                    assert_eq!(user_id, user2.id, "Should receive notification about user2 joining");
                }
                _ => panic!("Expected GroupEvent::Joined, got: {:?}", event),
            },
            _ => panic!("Expected Event message, got: {:?}", notification1),
        }

        // Connect user3 and join
        let (mut ws3, _) = connect_chat_websocket_with_auth(addr, &token3)
            .await
            .expect("Failed to connect user3 to websocket");

        let join_id3 = Uuid::new_v4().to_string();
        let join_msg3 = WebSocketMessage::Request {
            request_id: join_id3.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response3 = send_websocket_message_and_get_response(&mut ws3, join_msg3)
            .await
            .expect("User3 failed to join group");

        // Assert: Both user1 and user2 should receive notification about user3 joining
        let notification_to_user1 = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have received a message");

        let notification_to_user2 = receive_websocket_message_with_timeout(&mut ws2, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have received a message");

        // Check notification to user1
        match notification_to_user1 {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::Joined { user_id }) => {
                    assert_eq!(user_id, user3.id, "User1 should receive notification about user3 joining");
                }
                _ => panic!("Expected GroupEvent::Joined, got: {:?}", event),
            },
            _ => panic!("Expected Event message, got: {:?}", notification_to_user1),
        }

        // Check notification to user2
        match notification_to_user2 {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::Joined { user_id }) => {
                    assert_eq!(user_id, user3.id, "User2 should receive notification about user3 joining");
                }
                _ => panic!("Expected GroupEvent::Joined, got: {:?}", event),
            },
            _ => panic!("Expected Event message, got: {:?}", notification_to_user2),
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id, user2.id, user3.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id, user3.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_no_notification_when_user_has_multiple_connections() {
        // Arrange: Create two users in the same group
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_multi_conn_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_multi_conn_user2".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_multi_conn_group", user1.id).await;
        
        // Add user2 to the same group
        let _membership2 = crate::common::add_test_user_to_a_group(user2.id, &group).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user1 and join the group
        let (mut ws1, _) = connect_chat_websocket_with_auth(addr, &token1)
            .await
            .expect("Failed to connect user1 to websocket");

        let join_id1 = Uuid::new_v4().to_string();
        let join_msg1 = WebSocketMessage::Request {
            request_id: join_id1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response1 = send_websocket_message_and_get_response(&mut ws1, join_msg1)
            .await
            .expect("User1 failed to join group");

        // Connect user2 with first connection and join
        let (mut ws2_conn1, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 first connection to websocket");

        let join_id2_conn1 = Uuid::new_v4().to_string();
        let join_msg2_conn1 = WebSocketMessage::Request {
            request_id: join_id2_conn1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2_conn1 = send_websocket_message_and_get_response(&mut ws2_conn1, join_msg2_conn1)
            .await
            .expect("User2 first connection failed to join group");

        // Clear the join notification for user2's first connection
        let _ = receive_websocket_message_with_timeout(&mut ws1, Duration::from_millis(500)).await;

        // Act: Connect user2 with second connection and join
        let (mut ws2_conn2, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 second connection to websocket");

        let join_id2_conn2 = Uuid::new_v4().to_string();
        let join_msg2_conn2 = WebSocketMessage::Request {
            request_id: join_id2_conn2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2_conn2 = send_websocket_message_and_get_response(&mut ws2_conn2, join_msg2_conn2)
            .await
            .expect("User2 second connection failed to join group");

        // Assert: User1 should NOT receive a notification because user2 already had a connection
        let no_notification = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(2))
            .await
            .expect("Should not error");

        assert!(no_notification.is_none(), "Should not receive notification when user already has active connections");

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id, user2.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_no_notification_when_users_not_connected() {
        // Arrange: Create two users in different groups (not connected)
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_not_connected_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_not_connected_user2".to_string()).await;
        
        // Create separate groups for each user
        let group1 = create_test_group_chat_with_invitation_and_membership("e2e_not_connected_group1", user1.id).await;
        let group2 = create_test_group_chat_with_invitation_and_membership("e2e_not_connected_group2", user2.id).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user1 and join their group
        let (mut ws1, _) = connect_chat_websocket_with_auth(addr, &token1)
            .await
            .expect("Failed to connect user1 to websocket");

        let join_id1 = Uuid::new_v4().to_string();
        let join_msg1 = WebSocketMessage::Request {
            request_id: join_id1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response1 = send_websocket_message_and_get_response(&mut ws1, join_msg1)
            .await
            .expect("User1 failed to join group");

        // Act: Connect user2 and join their separate group
        let (mut ws2, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 to websocket");

        let join_id2 = Uuid::new_v4().to_string();
        let join_msg2 = WebSocketMessage::Request {
            request_id: join_id2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2 = send_websocket_message_and_get_response(&mut ws2, join_msg2)
            .await
            .expect("User2 failed to join group");

        // Assert: User1 should NOT receive a notification because they're not connected to user2
        let no_notification = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(2))
            .await
            .expect("Should not error");

        assert!(no_notification.is_none(), "Should not receive notification from unconnected users");

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id], group1.id).await;
        cleanup_test_users_from_a_group_chat(vec![user2.id], group2.id).await;
        cleanup_group_chat(group1.id).await;
        cleanup_group_chat(group2.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_user_leave_with_remaining_connections_no_notification() {
        // Arrange: Create two users where one has multiple connections
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_leave_multi_conn_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_leave_multi_conn_user2".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_leave_multi_conn_group", user1.id).await;
        
        // Add user2 to the same group
        let _membership2 = crate::common::add_test_user_to_a_group(user2.id, &group).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user1 and join the group
        let (mut ws1, _) = connect_chat_websocket_with_auth(addr, &token1)
            .await
            .expect("Failed to connect user1 to websocket");

        let join_id1 = Uuid::new_v4().to_string();
        let join_msg1 = WebSocketMessage::Request {
            request_id: join_id1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response1 = send_websocket_message_and_get_response(&mut ws1, join_msg1)
            .await
            .expect("User1 failed to join group");

        // Connect user2 with two connections and join both
        let (mut ws2_conn1, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 first connection to websocket");

        let join_id2_conn1 = Uuid::new_v4().to_string();
        let join_msg2_conn1 = WebSocketMessage::Request {
            request_id: join_id2_conn1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2_conn1 = send_websocket_message_and_get_response(&mut ws2_conn1, join_msg2_conn1)
            .await
            .expect("User2 first connection failed to join group");

        // Clear the join notification
        let _ = receive_websocket_message_with_timeout(&mut ws1, Duration::from_millis(2000)).await;

        let (mut ws2_conn2, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 second connection to websocket");

        let join_id2_conn2 = Uuid::new_v4().to_string();
        let join_msg2_conn2 = WebSocketMessage::Request {
            request_id: join_id2_conn2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2_conn2 = send_websocket_message_and_get_response(&mut ws2_conn2, join_msg2_conn2)
            .await
            .expect("User2 second connection failed to join group");

        // Act: User2's first connection leaves the group
        let leave_id2_conn1 = Uuid::new_v4().to_string();
        let leave_msg2_conn1 = WebSocketMessage::Request {
            request_id: leave_id2_conn1.clone(),
            action: ClientAction::Groups(GroupAction::Leave {}),
        };
        let _response2_leave = send_websocket_message_and_get_response(&mut ws2_conn1, leave_msg2_conn1)
            .await
            .expect("User2 first connection failed to leave group");

        // Assert: User1 should NOT receive a left notification because user2 still has another connection
        let no_notification = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(2))
            .await
            .expect("Should not error");

        assert!(no_notification.is_none(), "Should not receive leave notification when user still has active connections");

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id, user2.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }
}
