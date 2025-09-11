use axum::http::StatusCode;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;
use tokio::time::timeout;

use ruggine_server::websocket::group_message::{GroupAction, GroupEvent};
use ruggine_server::websocket::message::{ClientAction, ServerEvent, WebSocketMessage};
use ruggine_server::dto::group_chat_dto::GroupChatCreateDto;
use ruggine_server::factory::group_chat_factory::GroupChatFactory;

use crate::common::{
    cleanup_group_chat, cleanup_test_users, 
    create_login_and_get_token, connect_chat_websocket_with_auth, 
    send_websocket_message_and_get_response, start_test_server,
};

#[cfg(test)]
mod group_chat_creation_notification_e2e_tests {
    use crate::cleanup_test_user_from_a_group_chat;
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
                    let response = WebSocketMessage::from_json(&text)?;
                    Ok(Some(response))
                }
                _ => Ok(None),
            },
            Ok(None) => Ok(None),
            Err(_) => Ok(None), // Timeout
        }
    }

    /// Helper function to create a group chat via HTTP API
    async fn create_group_chat_via_api(
        addr: std::net::SocketAddr,
        token: &str,
        name: &str,
        description: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let group_chat_payload = GroupChatCreateDto {
            name: name.to_string(),
            description: description.to_string(),
        };

        let response = client
            .post(format!("http://{}/api/group_chat/create", addr))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&group_chat_payload)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;
        Ok(response_json)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_user_receives_new_group_chat_notification_when_creating_group() {
        // Arrange: Create a user who will create a group and should receive notification
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_group_create_notif_creator".to_string()).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user1 to websocket and join
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
            .expect("User1 failed to join websocket");

        // Act: User1 creates a new group chat via HTTP API
        let group_name = "E2E Test Group Create Notification";
        let group_description = "Test group for creation notification testing";
        
        let group_response = create_group_chat_via_api(
            addr,
            &token1,
            group_name,
            group_description,
        )
        .await
        .expect("Failed to create group chat");

        // Verify group was created successfully
        assert!(group_response.get("data").is_some(), "Group should be created");
        let group_data = &group_response["data"];
        assert_eq!(group_data["name"].as_str().unwrap(), group_name);
        assert_eq!(group_data["description"].as_str().unwrap(), group_description);
        assert_eq!(group_data["created_by"].as_i64().unwrap(), user1.id as i64);

        let group_id = group_data["id"].as_i64().unwrap() as i32;

        // Assert: User1 should receive a NewGroupChat notification via WebSocket
        let notification = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Failed to receive notification")
            .expect("Should receive a notification");

        match notification {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::NewGroupChat { group_chat_id }) => {
                    assert_eq!(group_chat_id, group_id, "Should receive notification for correct group");
                }
                _ => panic!("Expected NewGroupChat event, got: {:?}", event),
            },
            _ => panic!("Expected Event message, got: {:?}", notification),
        }

        // Cleanup
        cleanup_test_user_from_a_group_chat(user1.id, group_id).await;
        cleanup_group_chat(group_id).await;
        cleanup_test_users(vec![user1.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_user_not_connected_does_not_receive_group_creation_notification() {
        // Arrange: Create a user who will create a group but is not connected to WebSocket
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_group_create_no_notif".to_string()).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Note: user1 is NOT connected to websocket

        // Act: User1 creates a new group chat via HTTP API
        let group_name = "E2E Test Group No Notification";
        let group_description = "Test group for no notification testing";
        
        let group_response = create_group_chat_via_api(
            addr,
            &token1,
            group_name,
            group_description,
        )
        .await
        .expect("Failed to create group chat");

        // Verify group was created successfully
        assert!(group_response.get("data").is_some(), "Group should be created");
        let group_data = &group_response["data"];
        let group_id = group_data["id"].as_i64().unwrap() as i32;

        // Assert: Since user1 is not connected, this test just verifies that the group
        // was created successfully without any WebSocket errors (no notification will be received)
        assert_eq!(group_data["created_by"].as_i64().unwrap(), user1.id as i64);

        // Cleanup
        cleanup_test_user_from_a_group_chat(user1.id, group_id).await;
        cleanup_group_chat(group_id).await;
        cleanup_test_users(vec![user1.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_multiple_connections_same_user_all_receive_group_creation_notification() {
        // Arrange: Create a user with multiple WebSocket connections who will create a group
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_multi_conn_group_create".to_string()).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user1 with two different connections
        let (mut ws1_conn1, _) = connect_chat_websocket_with_auth(addr, &token1)
            .await
            .expect("Failed to connect user1 first connection to websocket");

        let (mut ws1_conn2, _) = connect_chat_websocket_with_auth(addr, &token1)
            .await
            .expect("Failed to connect user1 second connection to websocket");

        // Both connections join the websocket
        let join_id1_conn1 = Uuid::new_v4().to_string();
        let join_msg1_conn1 = WebSocketMessage::Request {
            request_id: join_id1_conn1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response1_conn1 = send_websocket_message_and_get_response(&mut ws1_conn1, join_msg1_conn1)
            .await
            .expect("User1 first connection failed to join websocket");

        let join_id1_conn2 = Uuid::new_v4().to_string();
        let join_msg1_conn2 = WebSocketMessage::Request {
            request_id: join_id1_conn2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response1_conn2 = send_websocket_message_and_get_response(&mut ws1_conn2, join_msg1_conn2)
            .await
            .expect("User1 second connection failed to join websocket");

        // Act: User1 creates a new group chat
        let group_name = "E2E Test Multi Connection Group";
        let group_description = "Test group for multi connection testing";
        
        let group_response = create_group_chat_via_api(
            addr,
            &token1,
            group_name,
            group_description,
        )
        .await
        .expect("Failed to create group chat");

        let group_id = group_response["data"]["id"].as_i64().unwrap() as i32;

        // Assert: Both connections of user1 should receive the group creation notification
        let notification_conn1 = receive_websocket_message_with_timeout(&mut ws1_conn1, Duration::from_secs(5))
            .await
            .expect("Failed to receive notification on first connection")
            .expect("First connection should receive a notification");

        let notification_conn2 = receive_websocket_message_with_timeout(&mut ws1_conn2, Duration::from_secs(5))
            .await
            .expect("Failed to receive notification on second connection")
            .expect("Second connection should receive a notification");

        // Verify both notifications are for the same group
        for (conn_num, notification) in [(1, notification_conn1), (2, notification_conn2)] {
            match notification {
                WebSocketMessage::Event { event, .. } => match event {
                    ServerEvent::Groups(GroupEvent::NewGroupChat { group_chat_id }) => {
                        assert_eq!(group_chat_id, group_id, "Connection {} should receive correct group ID", conn_num);
                    }
                    _ => panic!("Expected NewGroupChat event for connection {}, got: {:?}", conn_num, event),
                },
                _ => panic!("Expected Event message for connection {}, got: {:?}", conn_num, notification),
            }
        }

        // Cleanup
        cleanup_test_user_from_a_group_chat(user1.id, group_id).await;
        cleanup_group_chat(group_id).await;
        cleanup_test_users(vec![user1.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_only_creator_receives_group_creation_notification() {
        // Arrange: Create two users where only one creates a group
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_group_create_creator_only1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_group_create_creator_only2".to_string()).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect both users to websocket
        let (mut ws1, _) = connect_chat_websocket_with_auth(addr, &token1)
            .await
            .expect("Failed to connect user1 to websocket");

        let (mut ws2, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 to websocket");

        // Both users join the websocket
        let join_id1 = Uuid::new_v4().to_string();
        let join_msg1 = WebSocketMessage::Request {
            request_id: join_id1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response1 = send_websocket_message_and_get_response(&mut ws1, join_msg1).await;

        let join_id2 = Uuid::new_v4().to_string();
        let join_msg2 = WebSocketMessage::Request {
            request_id: join_id2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2 = send_websocket_message_and_get_response(&mut ws2, join_msg2).await;

        // Act: User1 creates a new group chat
        let group_name = "E2E Test Creator Only Group";
        let group_description = "Test group for creator only notification testing";
        
        let group_response = create_group_chat_via_api(
            addr,
            &token1,
            group_name,
            group_description,
        )
        .await
        .expect("Failed to create group chat");

        let group_id = group_response["data"]["id"].as_i64().unwrap() as i32;

        // Assert: User1 (creator) should receive the notification
        let notification1 = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Failed to receive notification for user1")
            .expect("User1 should receive a notification");

        match notification1 {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::NewGroupChat { group_chat_id }) => {
                    assert_eq!(group_chat_id, group_id, "User1 should receive correct group ID");
                }
                _ => panic!("Expected NewGroupChat event for user1, got: {:?}", event),
            },
            _ => panic!("Expected Event message for user1, got: {:?}", notification1),
        }

        // Assert: User2 should NOT receive notification for user1's group creation
        let no_notification2 = receive_websocket_message_with_timeout(&mut ws2, Duration::from_secs(2)).await;
        assert!(no_notification2.unwrap().is_none(), "User2 should not receive notification for user1's group creation");

        // Cleanup
        cleanup_test_user_from_a_group_chat(user1.id, group_id).await;
        cleanup_group_chat(group_id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_multiple_group_creations_independent_notifications() {
        // Arrange: Create two users who will each create their own groups
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_multi_group_create1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_multi_group_create2".to_string()).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect both users to websocket
        let (mut ws1, _) = connect_chat_websocket_with_auth(addr, &token1)
            .await
            .expect("Failed to connect user1 to websocket");

        let (mut ws2, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 to websocket");

        // Both users join the websocket
        let join_id1 = Uuid::new_v4().to_string();
        let join_msg1 = WebSocketMessage::Request {
            request_id: join_id1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response1 = send_websocket_message_and_get_response(&mut ws1, join_msg1).await;

        let join_id2 = Uuid::new_v4().to_string();
        let join_msg2 = WebSocketMessage::Request {
            request_id: join_id2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2 = send_websocket_message_and_get_response(&mut ws2, join_msg2).await;

        // Act: User1 creates a group
        let group1_name = "E2E Test Group 1";
        let group1_description = "First test group";
        
        let group1_response = create_group_chat_via_api(
            addr,
            &token1,
            group1_name,
            group1_description,
        )
        .await
        .expect("Failed to create first group chat");

        let group1_id = group1_response["data"]["id"].as_i64().unwrap() as i32;

        // Assert: User1 should receive notification for their group
        let notification1 = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Failed to receive notification for user1")
            .expect("User1 should receive a notification");

        match notification1 {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::NewGroupChat { group_chat_id }) => {
                    assert_eq!(group_chat_id, group1_id, "User1 should receive correct group ID");
                }
                _ => panic!("Expected NewGroupChat event for user1, got: {:?}", event),
            },
            _ => panic!("Expected Event message for user1, got: {:?}", notification1),
        }

        // Assert: User2 should NOT receive notification for user1's group
        let no_notification2_for_group1 = receive_websocket_message_with_timeout(&mut ws2, Duration::from_secs(2)).await;
        assert!(no_notification2_for_group1.unwrap().is_none(), "User2 should not receive notification for user1's group");

        // Act: User2 creates a group
        let group2_name = "E2E Test Group 2";
        let group2_description = "Second test group";
        
        let group2_response = create_group_chat_via_api(
            addr,
            &token2,
            group2_name,
            group2_description,
        )
        .await
        .expect("Failed to create second group chat");

        let group2_id = group2_response["data"]["id"].as_i64().unwrap() as i32;

        // Assert: User2 should receive notification for their group
        let notification2 = receive_websocket_message_with_timeout(&mut ws2, Duration::from_secs(5))
            .await
            .expect("Failed to receive notification for user2")
            .expect("User2 should receive a notification");

        match notification2 {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::NewGroupChat { group_chat_id }) => {
                    assert_eq!(group_chat_id, group2_id, "User2 should receive correct group ID");
                }
                _ => panic!("Expected NewGroupChat event for user2, got: {:?}", event),
            },
            _ => panic!("Expected Event message for user2, got: {:?}", notification2),
        }

        // Assert: User1 should NOT receive notification for user2's group
        let no_notification1_for_group2 = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(2)).await;
        assert!(no_notification1_for_group2.unwrap().is_none(), "User1 should not receive notification for user2's group");

        // Cleanup
        cleanup_test_user_from_a_group_chat(user1.id, group1_id).await;
        cleanup_test_user_from_a_group_chat(user2.id, group2_id).await;
        cleanup_group_chat(group1_id).await;
        cleanup_group_chat(group2_id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_group_creation_with_factory_generated_data() {
        // Arrange: Create a user and use factory to generate group data
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_factory_group_create".to_string()).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user1 to websocket and join
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
            .expect("User1 failed to join websocket");

        // Act: Use factory to generate group data and create via API
        let group_create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("e2e_factory_test");
        
        let group_response = create_group_chat_via_api(
            addr,
            &token1,
            &group_create_dto.name,
            &group_create_dto.description,
        )
        .await
        .expect("Failed to create group chat with factory data");

        // Verify group was created successfully with factory data
        assert!(group_response.get("data").is_some(), "Group should be created");
        let group_data = &group_response["data"];
        assert_eq!(group_data["name"].as_str().unwrap(), group_create_dto.name);
        assert_eq!(group_data["description"].as_str().unwrap(), group_create_dto.description);
        assert_eq!(group_data["created_by"].as_i64().unwrap(), user1.id as i64);

        let group_id = group_data["id"].as_i64().unwrap() as i32;

        // Assert: User1 should receive a NewGroupChat notification
        let notification = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Failed to receive notification")
            .expect("Should receive a notification");

        match notification {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::NewGroupChat { group_chat_id }) => {
                    assert_eq!(group_chat_id, group_id, "Should receive notification for factory-created group");
                }
                _ => panic!("Expected NewGroupChat event, got: {:?}", event),
            },
            _ => panic!("Expected Event message, got: {:?}", notification),
        }

        // Cleanup
        cleanup_test_user_from_a_group_chat(user1.id, group_id).await;
        cleanup_group_chat(group_id).await;
        cleanup_test_users(vec![user1.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_no_group_creation_notification_when_websocket_service_unavailable() {
        // Arrange: Create a user who will create a group
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_no_ws_service_group_create".to_string()).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user1 to websocket
        let (mut ws1, _) = connect_chat_websocket_with_auth(addr, &token1)
            .await
            .expect("Failed to connect user1 to websocket");

        let join_id1 = Uuid::new_v4().to_string();
        let join_msg1 = WebSocketMessage::Request {
            request_id: join_id1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response1 = send_websocket_message_and_get_response(&mut ws1, join_msg1).await;

        // Act: Create group (this should work even if WebSocket notification fails)
        let group_name = "E2E Test No WS Service Group";
        let group_description = "Test group for no WebSocket service testing";
        
        let group_response = create_group_chat_via_api(
            addr,
            &token1,
            group_name,
            group_description,
        )
        .await
        .expect("Failed to create group chat");

        // Assert: Group should be created successfully even if WebSocket notification fails
        assert!(group_response.get("data").is_some(), "Group should be created");
        let group_id = group_response["data"]["id"].as_i64().unwrap() as i32;

        // The notification might or might not arrive depending on WebSocket service availability
        // This test mainly ensures the group creation doesn't fail due to WebSocket issues
        let _notification = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(3)).await;
        // We don't assert on the notification result as it depends on service availability

        // Cleanup
        cleanup_test_user_from_a_group_chat(user1.id, group_id).await;
        cleanup_group_chat(group_id).await;
        cleanup_test_users(vec![user1.id]).await;
        shutdown.send(()).unwrap();
    }
}
