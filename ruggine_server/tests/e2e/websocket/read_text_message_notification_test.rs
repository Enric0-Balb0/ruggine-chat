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
    create_test_text_message, cleanup_text_message, mark_message_as_read,
    add_test_user_to_a_group,
};

#[cfg(test)]
mod read_text_message_notification_e2e_tests {
    use tokio::time::error::Elapsed;
    use tungstenite::Error;
    use crate::{mark_message_as_sent, mark_message_as_sent_and_read};
    use super::*;

    /// Helper function to receive a websocket message with timeout
    async fn receive_websocket_message_with_timeout(
        ws_stream: &mut tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        timeout_duration: Duration,
    ) -> Result<Option<WebSocketMessage>, Box<dyn std::error::Error>> {
        match timeout(timeout_duration, ws_stream.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                let message = WebSocketMessage::from_json(&text)?;
                Ok(Some(message))
            }
            Ok(Some(Err(e))) => Err(Box::new(e)),
            Ok(None) => Ok(None),
            Err(_) => Ok(None), // Timeout
            _ => {Ok(None)}
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_read_text_message_notification_to_user() {
        // Arrange: Create two users in the same group
        let (sender_user, _password1, sender_token) =
            create_login_and_get_token("e2e_read_msg_sender".to_string()).await;
        let (reader_user, _password2, reader_token) =
            create_login_and_get_token("e2e_read_msg_reader".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_read_msg_group", sender_user.id).await;
        
        // Add reader user to the same group
        let _membership2 = add_test_user_to_a_group(reader_user.id, &group).await;

        // Create a test message from sender
        let message = create_test_text_message(sender_user.id, group.id, Some("Test message to be read".to_string())).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect reader user to websocket and join group
        let (mut reader_ws, _) = connect_chat_websocket_with_auth(addr, &reader_token)
            .await
            .expect("Failed to connect reader to websocket");

        let join_id = Uuid::new_v4().to_string();
        let join_msg = WebSocketMessage::Request {
            request_id: join_id.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response = send_websocket_message_and_get_response(&mut reader_ws, join_msg)
            .await
            .expect("Reader failed to join group");

        // Act: Mark message as read through API
        let client = reqwest::Client::new();
        let read_payload = json!({
            "text_message_id": message.id
        });

        mark_message_as_sent(reader_user.id, message.id).await;

        let api_response = client
            .patch(format!("http://{}/api/text_message/update_read_at", addr))
            .bearer_auth(&reader_token)
            .json(&read_payload)
            .send()
            .await
            .expect("Failed to send read update request");

        // Assert: API call should succeed
        assert_eq!(api_response.status(), StatusCode::OK);

        // Assert: Reader should receive a NewReadTextMessage notification
        let notification = receive_websocket_message_with_timeout(&mut reader_ws, Duration::from_secs(5))
            .await
            .expect("Failed to receive notification")
            .expect("No notification received");

        match notification {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::NewReadTextMessage {
                        text_message_id,
                        group_id,
                        user_id,
                    }) => {
                        assert_eq!(text_message_id, message.id, "Message ID should match");
                        assert_eq!(group_id, group.id, "Group ID should match");
                        assert_eq!(user_id, reader_user.id, "User ID should match the reader");
                    }
                    _ => panic!("Expected NewReadTextMessage event, got: {:?}", event),
                }
            }
            _ => panic!("Expected Event message, got: {:?}", notification),
        }

        // Cleanup
        cleanup_text_message(message.id).await;
        cleanup_test_users_from_a_group_chat(vec![sender_user.id, reader_user.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![sender_user.id, reader_user.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_read_text_message_notification_multiple_connections() {
        // Arrange: Create user with multiple connections
        let (sender_user, _password1, sender_token) =
            create_login_and_get_token("e2e_read_msg_multi_sender".to_string()).await;
        let (reader_user, reader_password, reader_token1) =
            create_login_and_get_token("e2e_read_msg_multi_reader".to_string()).await;
        
        // Create second token for same reader user (simulating multiple devices)
        let reader_token2 = crate::common::login_and_get_token_for_user(&reader_user, &reader_password).await;

        let group = create_test_group_chat_with_invitation_and_membership("e2e_read_msg_multi_group", sender_user.id).await;
        
        // Add reader user to the group
        let _membership = add_test_user_to_a_group(reader_user.id, &group).await;

        // Create a test message
        let message = create_test_text_message(sender_user.id, group.id, Some("Test message for multiple connections".to_string())).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect reader user with first connection
        let (mut reader_ws1, _) = connect_chat_websocket_with_auth(addr, &reader_token1)
            .await
            .expect("Failed to connect reader with first token");

        let join_id1 = Uuid::new_v4().to_string();
        let join_msg1 = WebSocketMessage::Request {
            request_id: join_id1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response1 = send_websocket_message_and_get_response(&mut reader_ws1, join_msg1)
            .await
            .expect("Reader failed to join group with first connection");

        // Connect reader user with second connection
        let (mut reader_ws2, _) = connect_chat_websocket_with_auth(addr, &reader_token2)
            .await
            .expect("Failed to connect reader with second token");

        let join_id2 = Uuid::new_v4().to_string();
        let join_msg2 = WebSocketMessage::Request {
            request_id: join_id2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2 = send_websocket_message_and_get_response(&mut reader_ws2, join_msg2)
            .await
            .expect("Reader failed to join group with second connection");

        // Act: Mark message as read through API using first token
        let client = reqwest::Client::new();
        let read_payload = json!({
            "text_message_id": message.id
        });

        mark_message_as_sent(reader_user.id, message.id).await;

        let api_response = client
            .patch(format!("http://{}/api/text_message/update_read_at", addr))
            .bearer_auth(&reader_token1)
            .json(&read_payload)
            .send()
            .await
            .expect("Failed to send read update request");

        assert_eq!(api_response.status(), StatusCode::OK);

        // Create tasks to listen for notifications on both connections
        let handle1 = tokio::spawn(async move {
            let notification = receive_websocket_message_with_timeout(&mut reader_ws1, Duration::from_secs(5))
                .await
                .expect("Failed to receive notification on connection 1")
                .expect("No notification received on connection 1");

            match notification {
                WebSocketMessage::Event { event, .. } => {
                    match event {
                        ServerEvent::Groups(GroupEvent::NewReadTextMessage {
                            text_message_id,
                            group_id,
                            user_id,
                        }) => (text_message_id, group_id, user_id),
                        _ => panic!("Expected NewReadTextMessage event on connection 1"),
                    }
                }
                _ => panic!("Expected Event message on connection 1"),
            }
        });

        let handle2 = tokio::spawn(async move {
            let notification = receive_websocket_message_with_timeout(&mut reader_ws2, Duration::from_secs(5))
                .await
                .expect("Failed to receive notification on connection 2")
                .expect("No notification received on connection 2");

            match notification {
                WebSocketMessage::Event { event, .. } => {
                    match event {
                        ServerEvent::Groups(GroupEvent::NewReadTextMessage {
                            text_message_id,
                            group_id,
                            user_id,
                        }) => (text_message_id, group_id, user_id),
                        _ => panic!("Expected NewReadTextMessage event on connection 2"),
                    }
                }
                _ => panic!("Expected Event message on connection 2"),
            }
        });

        // Assert: Both connections should receive the notification
        let (msg_id1, group_id1, user_id1) = handle1.await.expect("Connection 1 task failed");
        let (msg_id2, group_id2, user_id2) = handle2.await.expect("Connection 2 task failed");

        // Verify all values match expected
        assert_eq!(msg_id1, message.id, "Message ID should match on connection 1");
        assert_eq!(group_id1, group.id, "Group ID should match on connection 1");
        assert_eq!(user_id1, reader_user.id, "User ID should match on connection 1");

        assert_eq!(msg_id2, message.id, "Message ID should match on connection 2");
        assert_eq!(group_id2, group.id, "Group ID should match on connection 2");
        assert_eq!(user_id2, reader_user.id, "User ID should match on connection 2");

        // Cleanup
        cleanup_text_message(message.id).await;
        cleanup_test_users_from_a_group_chat(vec![sender_user.id, reader_user.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![sender_user.id, reader_user.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_read_text_message_notification_only_to_reader() {
        // Arrange: Create three users in the same group
        let (sender_user, _password1, sender_token) =
            create_login_and_get_token("e2e_read_msg_only_sender".to_string()).await;
        let (reader_user, _password2, reader_token) =
            create_login_and_get_token("e2e_read_msg_only_reader".to_string()).await;
        let (other_user, _password3, other_token) =
            create_login_and_get_token("e2e_read_msg_only_other".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_read_msg_only_group", sender_user.id).await;
        
        // Add both users to the group
        let _membership1 = add_test_user_to_a_group(reader_user.id, &group).await;
        let _membership2 = add_test_user_to_a_group(other_user.id, &group).await;

        // Create a test message
        let message = create_test_text_message(sender_user.id, group.id, Some("Test message for selective notification".to_string())).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect reader user
        let (mut reader_ws, _) = connect_chat_websocket_with_auth(addr, &reader_token)
            .await
            .expect("Failed to connect reader to websocket");

        let reader_join_id = Uuid::new_v4().to_string();
        let reader_join_msg = WebSocketMessage::Request {
            request_id: reader_join_id.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _reader_response = send_websocket_message_and_get_response(&mut reader_ws, reader_join_msg)
            .await
            .expect("Reader failed to join group");

        // Connect other user
        let (mut other_ws, _) = connect_chat_websocket_with_auth(addr, &other_token)
            .await
            .expect("Failed to connect other user to websocket");

        let other_join_id = Uuid::new_v4().to_string();
        let other_join_msg = WebSocketMessage::Request {
            request_id: other_join_id.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _other_response = send_websocket_message_and_get_response(&mut other_ws, other_join_msg)
            .await
            .expect("Other user failed to join group");

        // Wait and drain any join notifications that were triggered by users joining
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Drain any pending join notifications for both WebSockets
        while let Ok(Some(_)) = tokio::time::timeout(Duration::from_millis(50), reader_ws.next()).await {
            // Consume any pending messages
        }
        while let Ok(Some(_)) = tokio::time::timeout(Duration::from_millis(50), other_ws.next()).await {
            // Consume any pending messages
        }

        // Act: Mark message as read by reader user through API
        let client = reqwest::Client::new();
        let read_payload = json!({
            "text_message_id": message.id
        });

        mark_message_as_sent(reader_user.id, message.id).await;

        let api_response = client
            .patch(format!("http://{}/api/text_message/update_read_at", addr))
            .bearer_auth(&reader_token)
            .json(&read_payload)
            .send()
            .await
            .expect("Failed to send read update request");

        assert_eq!(api_response.status(), StatusCode::OK);

        // Create tasks to listen for notifications
        let reader_handle = tokio::spawn(async move {
            let notification = receive_websocket_message_with_timeout(&mut reader_ws, Duration::from_secs(5))
                .await
                .expect("Failed to receive notification for reader")
                .expect("No notification received for reader");

            println!("Reader received notification: {:?}", notification);
            
            match notification {
                WebSocketMessage::Event { event, .. } => {
                    match event {
                        ServerEvent::Groups(GroupEvent::NewReadTextMessage { .. }) => {
                            println!("Reader correctly received NewReadTextMessage event");
                            true
                        },
                        _ => {
                            println!("Reader received unexpected event: {:?}", event);
                            false
                        }
                    }
                }
                _ => {
                    println!("Reader received non-event message: {:?}", notification);
                    false
                }
            }
        });

        let other_handle = tokio::spawn(async move {
            // Other user should NOT receive the notification
            let notification = receive_websocket_message_with_timeout(&mut other_ws, Duration::from_secs(2)).await;
            
            match notification {
                Ok(Some(WebSocketMessage::Event { event, .. })) => {
                    match event {
                        ServerEvent::Groups(GroupEvent::NewReadTextMessage { .. }) => {
                            println!("ERROR: Other user received NewReadTextMessage notification when they shouldn't have!");
                            false // This indicates the test should fail
                        }
                        _ => {
                            println!("Other user received acceptable event: {:?}", event);
                            true // Other events are acceptable
                        }
                    }
                }
                Ok(Some(_)) => {
                    println!("Other user received non-event message (acceptable)");
                    true // Non-event messages are acceptable
                }
                Ok(None) => {
                    println!("Other user correctly received no notification");
                    true // No notification is expected - this is correct
                }
                Err(e) => {
                    println!("Other user timeout/error (expected): {:?}", e);
                    true // Timeout is expected
                }
            }
        });

        // Assert: Reader should receive notification, other user should not
        let reader_received = reader_handle.await.expect("Reader task failed");
        let other_not_received = other_handle.await.expect("Other user task failed");

        assert!(reader_received, "Reader should receive NewReadTextMessage notification");
        assert!(other_not_received, "Other user should NOT receive NewReadTextMessage notification");

        // Cleanup
        cleanup_text_message(message.id).await;
        cleanup_test_users_from_a_group_chat(vec![sender_user.id, reader_user.id, other_user.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![sender_user.id, reader_user.id, other_user.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_read_text_message_notification_with_disconnected_user() {
        // Arrange: Create two users but only connect one
        let (sender_user, _password1, sender_token) =
            create_login_and_get_token("e2e_read_msg_disconnected_sender".to_string()).await;
        let (reader_user, _password2, reader_token) =
            create_login_and_get_token("e2e_read_msg_disconnected_reader".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_read_msg_disconnected_group", sender_user.id).await;
        
        // Add reader user to the group
        let _membership = add_test_user_to_a_group(reader_user.id, &group).await;

        // Create a test message
        let message = create_test_text_message(sender_user.id, group.id, Some("Test message for disconnected user".to_string())).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Note: Don't connect reader user to websocket

        // Act: Mark message as read through API (user is not connected to websocket)
        let client = reqwest::Client::new();
        let read_payload = json!({
            "text_message_id": message.id
        });

        mark_message_as_sent(reader_user.id, message.id).await;

        let api_response = client
            .patch(format!("http://{}/api/text_message/update_read_at", addr))
            .bearer_auth(&reader_token)
            .json(&read_payload)
            .send()
            .await
            .expect("Failed to send read update request");

        // Assert: API call should still succeed even without websocket connection
        assert_eq!(api_response.status(), StatusCode::OK);

        let response_json: serde_json::Value = api_response.json().await.unwrap();
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = &response_json["data"];
        assert_eq!(data["text_message_id"].as_i64().unwrap() as i32, message.id);
        assert_eq!(data["user_id"].as_i64().unwrap() as i32, reader_user.id);
        assert!(data["read_at"].is_string(), "read_at should be set");

        // Cleanup
        cleanup_text_message(message.id).await;
        cleanup_test_users_from_a_group_chat(vec![sender_user.id, reader_user.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![sender_user.id, reader_user.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_read_text_message_notification_already_read() {
        // Arrange: Create users and message, then mark as read beforehand
        let (sender_user, _password1, sender_token) =
            create_login_and_get_token("e2e_read_msg_already_sender".to_string()).await;
        let (reader_user, _password2, reader_token) =
            create_login_and_get_token("e2e_read_msg_already_reader".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_read_msg_already_group", sender_user.id).await;
        
        // Add reader user to the group
        let _membership = add_test_user_to_a_group(reader_user.id, &group).await;

        // Create a test message
        let message = create_test_text_message(sender_user.id, group.id, Some("Test message already read".to_string())).await;

        // Mark message as read using common utility
        mark_message_as_sent_and_read(reader_user.id, message.id).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Act: Try to mark message as read again through API
        let client = reqwest::Client::new();
        let read_payload = json!({
            "text_message_id": message.id
        });

        let api_response = client
            .patch(format!("http://{}/api/text_message/update_read_at", addr))
            .bearer_auth(&reader_token)
            .json(&read_payload)
            .send()
            .await
            .expect("Failed to send read update request");

        // Assert: API call should fail when message is already read
        assert_eq!(api_response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_text_message(message.id).await;
        cleanup_test_users_from_a_group_chat(vec![sender_user.id, reader_user.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![sender_user.id, reader_user.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_read_text_message_notification_unauthorized_user() {
        // Arrange: Create users where one is not in the group
        let (sender_user, _password1, sender_token) =
            create_login_and_get_token("e2e_read_msg_unauth_sender".to_string()).await;
        let (reader_user, _password2, reader_token) =
            create_login_and_get_token("e2e_read_msg_unauth_reader".to_string()).await;
        let (unauthorized_user, _password3, unauthorized_token) =
            create_login_and_get_token("e2e_read_msg_unauth_unauthorized".to_string()).await;
        
        let group = create_test_group_chat_with_invitation_and_membership("e2e_read_msg_unauth_group", sender_user.id).await;
        
        // Add only reader user to the group (not unauthorized_user)
        let _membership = add_test_user_to_a_group(reader_user.id, &group).await;

        // Create a test message
        let message = create_test_text_message(sender_user.id, group.id, Some("Test message for unauthorized access".to_string())).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Act: Try to mark message as read using unauthorized user token
        let client = reqwest::Client::new();
        let read_payload = json!({
            "text_message_id": message.id
        });

        let api_response = client
            .patch(format!("http://{}/api/text_message/update_read_at", addr))
            .bearer_auth(&unauthorized_token)
            .json(&read_payload)
            .send()
            .await
            .expect("Failed to send read update request");

        // Assert: API call should fail with forbidden status
        assert_eq!(api_response.status(), StatusCode::FORBIDDEN);

        // Cleanup
        cleanup_text_message(message.id).await;
        cleanup_test_users_from_a_group_chat(vec![sender_user.id, reader_user.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![sender_user.id, reader_user.id, unauthorized_user.id]).await;
        shutdown.send(()).unwrap();
    }
}