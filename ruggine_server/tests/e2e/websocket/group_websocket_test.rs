use axum::http::StatusCode;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

use ruggine_server::websocket::group_message::GroupAction;
use ruggine_server::websocket::message::{ClientAction, ControlMessage, WebSocketMessage};

use crate::common::{
    cleanup_group_chat, cleanup_test_users, cleanup_test_users_from_a_group_chat
    , create_login_and_get_token,
    create_test_group_chat_with_invitation_and_membership, create_test_users_for_a_group,
};

#[cfg(test)]
mod group_websocket_e2e_tests {
    use crate::{cleanup_text_message, connect_group_websocket_with_auth, send_websocket_message_and_get_response, start_test_server};
    use ruggine_server::websocket::GroupEvent::NewMessage;
    use std::time::Duration;
    use tower::ServiceExt;

    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_websocket_connection_with_valid_token() {
        // Arrange: Create user, group and get authentication token
        let (user, _password, token) =
            create_login_and_get_token("e2e_ws_valid_token".to_string()).await;
        let group =
            create_test_group_chat_with_invitation_and_membership("e2e_ws_group", user.id).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Act: Attempt to connect to websocket with valid token
        let result = connect_group_websocket_with_auth(addr, &token).await;

        // Assert: Connection should succeed
        if let Err(e) = &result {
            eprintln!("WebSocket connection error: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "WebSocket connection should succeed with valid token: {:?}",
            result.err()
        );
        let (_ws_stream, response) = result.unwrap();
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_websocket_connection_with_invalid_token() {
        // Arrange: Start test server
        let (addr, shutdown) = start_test_server().await;

        // Act: Attempt to connect to websocket with invalid token
        let result = connect_group_websocket_with_auth(addr, "invalid_token").await;

        // Assert: Connection should fail
        assert!(
            result.is_err(),
            "WebSocket connection should fail with invalid token"
        );
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_websocket_join_group_action() {
        // Arrange: Create user, group and get authentication token
        let (user, _password, token) =
            create_login_and_get_token("e2e_ws_join_group".to_string()).await;
        let group =
            create_test_group_chat_with_invitation_and_membership("e2e_ws_join_group", user.id)
                .await;

        // Start test server and establish websocket connection
        let (addr, shutdown) = start_test_server().await;

        let (mut ws_stream, _response) = connect_group_websocket_with_auth(addr, &token)
            .await
            .expect("Failed to connect to websocket");

        // Act: Send join group message
        let request_id = Uuid::new_v4().to_string();
        let join_message = WebSocketMessage::Request {
            request_id: request_id.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };

        let response = send_websocket_message_and_get_response(&mut ws_stream, join_message)
            .await
            .expect("Failed to send/receive websocket message");

        // Assert: Should receive success response
        match response {
            WebSocketMessage::Response {
                request_id: resp_id,
                ok,
                data: _,
                error,
            } => {
                assert_eq!(resp_id, request_id, "Response ID should match request ID");
                assert!(ok, "Join group should succeed");
                assert!(error.is_none(), "Should not have error");
            }
            _ => panic!("Expected Response message, got: {:?}", response),
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_websocket_leave_group_action() {
        // Arrange: Create user, group and get authentication token
        let (user, _password, token) =
            create_login_and_get_token("e2e_ws_leave_group".to_string()).await;
        let group =
            create_test_group_chat_with_invitation_and_membership("e2e_ws_leave_group", user.id)
                .await;

        // Start test server and establish websocket connection
        let (addr, shutdown) = start_test_server().await;

        let (mut ws_stream, _response) = connect_group_websocket_with_auth(addr, &token)
            .await
            .expect("Failed to connect to websocket");

        // First join the group
        let join_request_id = Uuid::new_v4().to_string();
        let join_message = WebSocketMessage::Request {
            request_id: join_request_id.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };

        let _join_response = send_websocket_message_and_get_response(&mut ws_stream, join_message)
            .await
            .expect("Failed to join group");

        // Act: Send leave group message
        let leave_request_id = Uuid::new_v4().to_string();
        let leave_message = WebSocketMessage::Request {
            request_id: leave_request_id.clone(),
            action: ClientAction::Groups(GroupAction::Leave {}),
        };

        let response = send_websocket_message_and_get_response(&mut ws_stream, leave_message)
            .await
            .expect("Failed to send/receive websocket message");

        // Assert: Should receive success response
        match response {
            WebSocketMessage::Response {
                request_id: resp_id,
                ok,
                data: _,
                error,
            } => {
                assert_eq!(
                    resp_id, leave_request_id,
                    "Response ID should match request ID"
                );
                assert!(ok, "Leave group should succeed");
                assert!(error.is_none(), "Should not have error");
            }
            _ => panic!("Expected Response message, got: {:?}", response),
        }

        // Connection should be closed after leaving
        let next_msg = ws_stream.next().await;
        assert!(
            next_msg.is_none() || matches!(next_msg, Some(Err(_))),
            "Connection should be closed after leaving"
        );

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_websocket_ping_pong() {
        // Arrange: Create user, group and get authentication token
        let (user, _password, token) =
            create_login_and_get_token("e2e_ws_ping_pong".to_string()).await;
        let group =
            create_test_group_chat_with_invitation_and_membership("e2e_ws_ping_pong", user.id)
                .await;

        // Start test server and establish websocket connection
        let (addr, shutdown) = start_test_server().await;

        let (mut ws_stream, _response) = connect_group_websocket_with_auth(addr, &token)
            .await
            .expect("Failed to connect to websocket");

        // Act: Send ping message
        let ping_message = WebSocketMessage::Control(ControlMessage::Ping);
        let response = send_websocket_message_and_get_response(&mut ws_stream, ping_message)
            .await
            .expect("Failed to send/receive websocket message");

        // Assert: Should receive pong response
        match response {
            WebSocketMessage::Control(ControlMessage::Pong) => {
                // Success case
            }
            _ => panic!("Expected Pong control message, got: {:?}", response),
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_websocket_multiple_users_join_same_group() {
        // Arrange: Create group owner and additional users
        let (owner, _password, owner_token) =
            create_login_and_get_token("e2e_ws_multi_owner".to_string()).await;
        let group =
            create_test_group_chat_with_invitation_and_membership("e2e_ws_multi_group", owner.id)
                .await;

        // Create additional users and add them to the group
        let test_users = create_test_users_for_a_group("e2e_ws_multi_user", 2, &group).await;
        let mut user_tokens = Vec::new();

        for (user, password, _membership) in &test_users {
            let token = crate::common::login_and_get_token_for_user(user, password).await;
            user_tokens.push(token);
        }

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Act: Connect all users to websocket and join group
        let mut connections = Vec::new();

        // Connect owner
        let (mut owner_ws, _) = connect_group_websocket_with_auth(addr, &owner_token)
            .await
            .expect("Failed to connect owner to websocket");

        let owner_join_id = Uuid::new_v4().to_string();
        let owner_join_msg = WebSocketMessage::Request {
            request_id: owner_join_id.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _owner_response =
            send_websocket_message_and_get_response(&mut owner_ws, owner_join_msg)
                .await
                .expect("Owner failed to join group");

        connections.push(owner_ws);

        // Connect other users
        for token in &user_tokens {
            let (mut user_ws, _) = connect_group_websocket_with_auth(addr, token)
                .await
                .expect("Failed to connect user to websocket");

            let user_join_id = Uuid::new_v4().to_string();
            let user_join_msg = WebSocketMessage::Request {
                request_id: user_join_id.clone(),
                action: ClientAction::Groups(GroupAction::Join {}),
            };
            let user_response =
                send_websocket_message_and_get_response(&mut user_ws, user_join_msg)
                    .await
                    .expect("User failed to join group");

            // Assert: Each user should successfully join
            match user_response {
                WebSocketMessage::Response { ok, error, .. } => {
                    assert!(ok, "User should successfully join group");
                    assert!(error.is_none(), "Should not have error");
                }
                _ => panic!("Expected Response message"),
            }

            connections.push(user_ws);
        }

        // Assert: All connections should be established
        assert_eq!(
            connections.len(),
            3,
            "Should have 3 active connections (owner + 2 users)"
        );

        // Cleanup
        let user_ids: Vec<i32> = test_users.iter().map(|(user, _, _)| user.id).collect();
        let mut all_user_ids = user_ids;
        all_user_ids.push(owner.id);

        cleanup_test_users_from_a_group_chat(all_user_ids.clone(), group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(all_user_ids).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_websocket_invalid_message_format() {
        // Arrange: Create user, group and get authentication token
        let (user, _password, token) =
            create_login_and_get_token("e2e_ws_invalid_msg".to_string()).await;
        let group =
            create_test_group_chat_with_invitation_and_membership("e2e_ws_invalid_msg", user.id)
                .await;

        // Start test server and establish websocket connection
        let (addr, shutdown) = start_test_server().await;

        let (mut ws_stream, _response) = connect_group_websocket_with_auth(addr, &token)
            .await
            .expect("Failed to connect to websocket");

        // Act: Send invalid JSON message
        let invalid_json = "{ invalid json }";
        ws_stream
            .send(Message::Text(invalid_json.to_string()))
            .await
            .expect("Failed to send message");

        // The connection should handle the error gracefully or close
        // We expect either an error response or connection closure
        if let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Should receive an error response
                    let response = WebSocketMessage::from_json(&text);
                    assert!(
                        response.is_err()
                            || matches!(
                                response.unwrap(),
                                WebSocketMessage::Control(ControlMessage::ValidationError { .. })
                            ),
                        "Should receive validation error or fail to parse"
                    );
                }
                Err(_) => {
                    // Connection closed due to invalid message - acceptable behavior
                }
                _ => panic!("Unexpected message type"),
            }
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_websocket_unsupported_action() {
        // Arrange: Create user, group and get authentication token
        let (user, _password, token) =
            create_login_and_get_token("e2e_ws_unsupported".to_string()).await;
        let group =
            create_test_group_chat_with_invitation_and_membership("e2e_ws_unsupported", user.id)
                .await;

        // Start test server and establish websocket connection
        let (addr, shutdown) = start_test_server().await;

        let (mut ws_stream, _response) = connect_group_websocket_with_auth(addr, &token)
            .await
            .expect("Failed to connect to websocket");

        // Act: Send test action (which should be unhandled)
        let request_id = Uuid::new_v4().to_string();
        let test_message = WebSocketMessage::Request {
            request_id: request_id.clone(),
            action: ClientAction::Test {
                message: "test".to_string(),
            },
        };

        // Send the message - it might not get a response since it's unhandled
        let message_json = test_message.to_json().expect("Failed to serialize message");
        ws_stream
            .send(Message::Text(message_json))
            .await
            .expect("Failed to send message");

        // The server should handle unhandled messages gracefully
        // It might not respond at all, or might send an error

        // Try to send a valid ping to ensure connection is still alive
        let ping_message = WebSocketMessage::Control(ControlMessage::Ping);
        let response = send_websocket_message_and_get_response(&mut ws_stream, ping_message).await;

        // Assert: Connection should still be alive and respond to ping
        assert!(
            response.is_ok(),
            "Connection should still be alive after unhandled message"
        );
        match response.unwrap() {
            WebSocketMessage::Control(ControlMessage::Pong) => {
                // Success - connection is still alive
            }
            _ => panic!("Expected Pong response to ping"),
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_websocket_connection_without_token() {
        // Arrange: Start test server
        let (addr, shutdown) = start_test_server().await;

        // Act: Attempt to connect to websocket without token
        let ws_url = format!("ws://{}/api/ws/group", addr);
        let result = connect_async(&ws_url).await;

        // Assert: Connection should fail due to missing authentication
        assert!(
            result.is_err(),
            "WebSocket connection should fail without token"
        );
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_websocket_multiple_receive_new_messages() {
        // Arrange: Create group owner and additional users
        let (owner, _password, owner_token) =
            create_login_and_get_token("e2e_ws_new_messages".to_string()).await;
        let group =
            create_test_group_chat_with_invitation_and_membership("e2e_ws_new_messages", owner.id)
                .await;

        // Create additional users and add them to the group
        let test_users = create_test_users_for_a_group("e2e_ws_messages", 2, &group).await;
        let mut user_tokens = Vec::new();

        for (user, password, _membership) in &test_users {
            let token = crate::common::login_and_get_token_for_user(user, password).await;
            user_tokens.push(token);
        }

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect all users to websocket and join group
        let mut connections = Vec::new();

        // Connect owner
        let (mut owner_ws, _) = connect_group_websocket_with_auth(addr, &owner_token)
            .await
            .expect("Failed to connect owner to websocket");

        let owner_join_id = Uuid::new_v4().to_string();
        let owner_join_msg = WebSocketMessage::Request {
            request_id: owner_join_id.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _owner_response =
            send_websocket_message_and_get_response(&mut owner_ws, owner_join_msg)
                .await
                .expect("Owner failed to join group");

        connections.push(owner_ws);

        // Connect other users
        for token in &user_tokens {
            let (mut user_ws, _) = connect_group_websocket_with_auth(addr, token)
                .await
                .expect("Failed to connect user to websocket");

            let user_join_id = Uuid::new_v4().to_string();
            let user_join_msg = WebSocketMessage::Request {
                request_id: user_join_id.clone(),
                action: ClientAction::Groups(GroupAction::Join {}),
            };
            let user_response =
                send_websocket_message_and_get_response(&mut user_ws, user_join_msg)
                    .await
                    .expect("User failed to join group");

            // Assert: Each user should successfully join
            match user_response {
                WebSocketMessage::Response { ok, error, .. } => {
                    assert!(ok, "User should successfully join group");
                    assert!(error.is_none(), "Should not have error");
                }
                _ => panic!("Expected Response message"),
            }

            connections.push(user_ws);
        }

        // Act and Assert: Send a message through the api and wait for the ws to receive it
        let mut handles = Vec::new();
        let payload = json!({
            "content": "Test message from e2e test",
            "group_chat_id": group.id,
        });

        // Create task for receiving messages
        for mut user_ws in connections {
            let handle = tokio::spawn(async move {
                while let Some(message) = user_ws.next().await {
                    // Handle incoming WebSocket messages
                    match message.unwrap() {
                        Message::Text(text) => {
                            let parsed: WebSocketMessage = serde_json::from_str(&text).unwrap();
                            match parsed {
                                WebSocketMessage::Event { event, timestamp } => {
                                    // Success
                                    match event {
                                        ruggine_server::websocket::ServerEvent::Groups(NewMessage {
                                            message_id,
                                            group_id,
                                            sender_id,
                                            sender_username,
                                            content,
                                            sent_at,
                                        }) => {
                                            assert_eq!(content, "Test message from e2e test");
                                            break; // Exit after receiving the expected message
                                        }
                                        _ => {
                                            panic!("Unexpected event type");
                                        }
                                    }
                                }
                                _ => {
                                    panic!("Unexpected WS message type");
                                }
                            }
                        }
                        _ => {
                            panic!("Unexpected WS message type");
                        }
                    }
                }
            });
            handles.push(handle);
        }

        // Sleep for 100ms
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create task for sending message
        let token = owner_token.clone();
        let payload_clone = payload.clone();
        let message_id = tokio::spawn(async move {
            let client = reqwest::Client::new();

            let response = client.post(format!("http://{}/api/text_message/create", addr))
                .bearer_auth(&token)
                .json(&payload_clone)
                .send()
                .await
                .unwrap();

            assert_eq!(response.status(), StatusCode::OK);

            let response_json: serde_json::Value = response.json().await.unwrap();

            // Verify response structure
            assert!(response_json.get("data").is_some(), "Response should contain data field");

            let message_data = &response_json["data"];
            let message_id = message_data["id"].as_i64().unwrap() as i32;
            message_id
        }).await.unwrap();

        // Join all handles
        for handle in handles {
            let _ = handle.await;
        }

        // Cleanup
        let user_ids: Vec<i32> = test_users.iter().map(|(user, _, _)| user.id).collect();
        let mut all_user_ids = user_ids;
        all_user_ids.push(owner.id);

        cleanup_text_message(message_id).await;
        cleanup_test_users_from_a_group_chat(all_user_ids.clone(), group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(all_user_ids).await;
        shutdown.send(()).unwrap();
    }
}
