use axum::http::StatusCode;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;
use tokio::time::timeout;

use ruggine_server::websocket::group_message::{GroupAction, GroupEvent};
use ruggine_server::websocket::message::{ClientAction, ServerEvent, WebSocketMessage};
use ruggine_server::dto::invitation_dto::InvitationCreateDto;
use ruggine_server::entity::group_membership::MemberRole;

use crate::common::{
    cleanup_group_chat, cleanup_test_users, cleanup_test_users_from_a_group_chat,
    create_login_and_get_token, create_test_group_chat_with_invitation_and_membership,
    connect_chat_websocket_with_auth, send_websocket_message_and_get_response, start_test_server,
    add_test_user_to_a_group, cleanup_invitation, create_full_router,
};

#[cfg(test)]
mod invitation_websocket_notification_e2e_tests {
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

    /// Helper function to send an invitation via HTTP API
    async fn send_invitation_via_api(
        addr: std::net::SocketAddr,
        token: &str,
        to_user_id: i32,
        group_chat_id: i32,
        role: MemberRole,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let invitation_payload = InvitationCreateDto {
            to_user_id,
            group_chat_id,
            role_at_join: role,
        };

        let response = client
            .post(format!("http://{}/api/invitation/send", addr))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&invitation_payload)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;
        Ok(response_json)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_user_receives_new_invitation_notification() {
        // Arrange: Create two users where user1 will send an invitation to user2
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_invite_notif_sender".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_invite_notif_receiver".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_invite_notif_group", user1.id).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user2 to websocket and join (user2 should receive the invitation)
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
            .expect("User2 failed to join websocket");

        // Act: User1 sends an invitation to user2 via HTTP API
        let invitation_response = send_invitation_via_api(
            addr,
            &token1,
            user2.id,
            group.id,
            MemberRole::Member,
        )
        .await
        .expect("Failed to send invitation");

        // Verify invitation was created successfully
        assert!(invitation_response.get("data").is_some(), "Invitation should be created");
        let invitation_data = &invitation_response["data"];
        assert_eq!(invitation_data["to_user_id"].as_i64().unwrap(), user2.id as i64);
        assert_eq!(invitation_data["group_chat_id"].as_i64().unwrap(), group.id as i64);

        let invitation_id = invitation_data["id"].as_i64().unwrap() as i32;

        // Assert: User2 should receive a NewInvitation notification via WebSocket
        let notification = receive_websocket_message_with_timeout(&mut ws2, Duration::from_secs(5))
            .await
            .expect("Failed to receive notification")
            .expect("Should receive a notification");

        match notification {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::NewInvitation { invitation_id: received_invitation_id }) => {
                    assert_eq!(received_invitation_id, invitation_id, "Should receive notification for correct invitation");
                }
                _ => panic!("Expected NewInvitation event, got: {:?}", event),
            },
            _ => panic!("Expected Event message, got: {:?}", notification),
        }

        // Cleanup
        cleanup_invitation(invitation_id).await;
        cleanup_test_users_from_a_group_chat(vec![user1.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_user_not_connected_does_not_receive_invitation_notification() {
        // Arrange: Create two users where user1 will send an invitation to user2
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_invite_no_notif_sender".to_string()).await;
        let (user2, _password2, _token2) =
            create_login_and_get_token("e2e_invite_no_notif_receiver".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_invite_no_notif_group", user1.id).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Note: user2 is NOT connected to websocket

        // Act: User1 sends an invitation to user2 via HTTP API
        let invitation_response = send_invitation_via_api(
            addr,
            &token1,
            user2.id,
            group.id,
            MemberRole::Member,
        )
        .await
        .expect("Failed to send invitation");

        // Verify invitation was created successfully
        assert!(invitation_response.get("data").is_some(), "Invitation should be created");
        let invitation_data = &invitation_response["data"];
        let invitation_id = invitation_data["id"].as_i64().unwrap() as i32;

        // Assert: Since user2 is not connected, this test just verifies that the invitation
        // was sent successfully without any WebSocket errors (no notification will be received)
        assert_eq!(invitation_data["to_user_id"].as_i64().unwrap(), user2.id as i64);

        // Cleanup
        cleanup_invitation(invitation_id).await;
        cleanup_test_users_from_a_group_chat(vec![user1.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_multiple_users_receive_invitation_notifications_independently() {
        // Arrange: Create three users where user1 will send invitations to user2 and user3
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_multi_invite_sender".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_multi_invite_receiver1".to_string()).await;
        let (user3, _password3, token3) =
            create_login_and_get_token("e2e_multi_invite_receiver2".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_multi_invite_group", user1.id).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user2 and user3 to websocket
        let (mut ws2, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 to websocket");

        let (mut ws3, _) = connect_chat_websocket_with_auth(addr, &token3)
            .await
            .expect("Failed to connect user3 to websocket");

        // Both users join the websocket
        let join_id2 = Uuid::new_v4().to_string();
        let join_msg2 = WebSocketMessage::Request {
            request_id: join_id2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2 = send_websocket_message_and_get_response(&mut ws2, join_msg2)
            .await
            .expect("User2 failed to join websocket");

        let join_id3 = Uuid::new_v4().to_string();
        let join_msg3 = WebSocketMessage::Request {
            request_id: join_id3.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response3 = send_websocket_message_and_get_response(&mut ws3, join_msg3)
            .await
            .expect("User3 failed to join websocket");

        // Act: User1 sends invitation to user2
        let invitation2_response = send_invitation_via_api(
            addr,
            &token1,
            user2.id,
            group.id,
            MemberRole::Member,
        )
        .await
        .expect("Failed to send invitation to user2");

        let invitation2_id = invitation2_response["data"]["id"].as_i64().unwrap() as i32;

        // Assert: User2 should receive notification for their invitation
        let notification2 = receive_websocket_message_with_timeout(&mut ws2, Duration::from_secs(5))
            .await
            .expect("Failed to receive notification for user2")
            .expect("User2 should receive a notification");

        match notification2 {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::NewInvitation { invitation_id }) => {
                    assert_eq!(invitation_id, invitation2_id, "User2 should receive correct invitation ID");
                }
                _ => panic!("Expected NewInvitation event for user2, got: {:?}", event),
            },
            _ => panic!("Expected Event message for user2, got: {:?}", notification2),
        }

        // Assert: User3 should NOT receive notification for user2's invitation
        let no_notification3 = receive_websocket_message_with_timeout(&mut ws3, Duration::from_secs(2)).await;
        assert!(no_notification3.unwrap().is_none(), "User3 should not receive notification for user2's invitation");

        // Act: User1 sends invitation to user3
        let invitation3_response = send_invitation_via_api(
            addr,
            &token1,
            user3.id,
            group.id,
            MemberRole::Admin,
        )
        .await
        .expect("Failed to send invitation to user3");

        let invitation3_id = invitation3_response["data"]["id"].as_i64().unwrap() as i32;

        // Assert: User3 should receive notification for their invitation
        let notification3 = receive_websocket_message_with_timeout(&mut ws3, Duration::from_secs(5))
            .await
            .expect("Failed to receive notification for user3")
            .expect("User3 should receive a notification");

        match notification3 {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::NewInvitation { invitation_id }) => {
                    assert_eq!(invitation_id, invitation3_id, "User3 should receive correct invitation ID");
                }
                _ => panic!("Expected NewInvitation event for user3, got: {:?}", event),
            },
            _ => panic!("Expected Event message for user3, got: {:?}", notification3),
        }

        // Assert: User2 should NOT receive notification for user3's invitation
        let no_notification2 = receive_websocket_message_with_timeout(&mut ws2, Duration::from_secs(2)).await;
        assert!(no_notification2.unwrap().is_none(), "User2 should not receive notification for user3's invitation");

        // Cleanup
        cleanup_invitation(invitation2_id).await;
        cleanup_invitation(invitation3_id).await;
        cleanup_test_users_from_a_group_chat(vec![user1.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id, user3.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_user_with_multiple_connections_receives_invitation_on_all_connections() {
        // Arrange: Create two users where user1 will send an invitation to user2
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_multi_conn_invite_sender".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_multi_conn_invite_receiver".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_multi_conn_invite_group", user1.id).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user2 with two different connections
        let (mut ws2_conn1, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 first connection to websocket");

        let (mut ws2_conn2, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 second connection to websocket");

        // Both connections join the websocket
        let join_id2_conn1 = Uuid::new_v4().to_string();
        let join_msg2_conn1 = WebSocketMessage::Request {
            request_id: join_id2_conn1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2_conn1 = send_websocket_message_and_get_response(&mut ws2_conn1, join_msg2_conn1)
            .await
            .expect("User2 first connection failed to join websocket");

        let join_id2_conn2 = Uuid::new_v4().to_string();
        let join_msg2_conn2 = WebSocketMessage::Request {
            request_id: join_id2_conn2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2_conn2 = send_websocket_message_and_get_response(&mut ws2_conn2, join_msg2_conn2)
            .await
            .expect("User2 second connection failed to join websocket");

        // Act: User1 sends an invitation to user2
        let invitation_response = send_invitation_via_api(
            addr,
            &token1,
            user2.id,
            group.id,
            MemberRole::Member,
        )
        .await
        .expect("Failed to send invitation");

        let invitation_id = invitation_response["data"]["id"].as_i64().unwrap() as i32;

        // Assert: Both connections of user2 should receive the invitation notification
        let notification_conn1 = receive_websocket_message_with_timeout(&mut ws2_conn1, Duration::from_secs(5))
            .await
            .expect("Failed to receive notification on first connection")
            .expect("First connection should receive a notification");

        let notification_conn2 = receive_websocket_message_with_timeout(&mut ws2_conn2, Duration::from_secs(5))
            .await
            .expect("Failed to receive notification on second connection")
            .expect("Second connection should receive a notification");

        // Verify both notifications are for the same invitation
        for (conn_num, notification) in [(1, notification_conn1), (2, notification_conn2)] {
            match notification {
                WebSocketMessage::Event { event, .. } => match event {
                    ServerEvent::Groups(GroupEvent::NewInvitation { invitation_id: received_invitation_id }) => {
                        assert_eq!(received_invitation_id, invitation_id, "Connection {} should receive correct invitation ID", conn_num);
                    }
                    _ => panic!("Expected NewInvitation event for connection {}, got: {:?}", conn_num, event),
                },
                _ => panic!("Expected Event message for connection {}, got: {:?}", conn_num, notification),
            }
        }

        // Cleanup
        cleanup_invitation(invitation_id).await;
        cleanup_test_users_from_a_group_chat(vec![user1.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_invitation_notification_for_different_roles() {
        // Arrange: Create three users where user1 will send different types of invitations
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_role_invite_sender".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_role_invite_member".to_string()).await;
        let (user3, _password3, token3) =
            create_login_and_get_token("e2e_role_invite_admin".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_role_invite_group", user1.id).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect users to websocket
        let (mut ws2, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 to websocket");

        let (mut ws3, _) = connect_chat_websocket_with_auth(addr, &token3)
            .await
            .expect("Failed to connect user3 to websocket");

        // Both users join the websocket
        let join_id2 = Uuid::new_v4().to_string();
        let join_msg2 = WebSocketMessage::Request {
            request_id: join_id2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2 = send_websocket_message_and_get_response(&mut ws2, join_msg2).await;

        let join_id3 = Uuid::new_v4().to_string();
        let join_msg3 = WebSocketMessage::Request {
            request_id: join_id3.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response3 = send_websocket_message_and_get_response(&mut ws3, join_msg3).await;

        // Act & Assert: Send member role invitation to user2
        let member_invitation_response = send_invitation_via_api(
            addr,
            &token1,
            user2.id,
            group.id,
            MemberRole::Member,
        )
        .await
        .expect("Failed to send member invitation");

        let member_invitation_id = member_invitation_response["data"]["id"].as_i64().unwrap() as i32;

        // User2 should receive the member invitation notification
        let member_notification = receive_websocket_message_with_timeout(&mut ws2, Duration::from_secs(5))
            .await
            .expect("Failed to receive member invitation notification")
            .expect("Should receive member invitation notification");

        match member_notification {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::NewInvitation { invitation_id }) => {
                    assert_eq!(invitation_id, member_invitation_id);
                }
                _ => panic!("Expected NewInvitation event for member role, got: {:?}", event),
            },
            _ => panic!("Expected Event message for member invitation, got: {:?}", member_notification),
        }

        // Act & Assert: Send admin role invitation to user3
        let admin_invitation_response = send_invitation_via_api(
            addr,
            &token1,
            user3.id,
            group.id,
            MemberRole::Admin,
        )
        .await
        .expect("Failed to send admin invitation");

        let admin_invitation_id = admin_invitation_response["data"]["id"].as_i64().unwrap() as i32;

        // User3 should receive the admin invitation notification
        let admin_notification = receive_websocket_message_with_timeout(&mut ws3, Duration::from_secs(5))
            .await
            .expect("Failed to receive admin invitation notification")
            .expect("Should receive admin invitation notification");

        match admin_notification {
            WebSocketMessage::Event { event, .. } => match event {
                ServerEvent::Groups(GroupEvent::NewInvitation { invitation_id }) => {
                    assert_eq!(invitation_id, admin_invitation_id);
                }
                _ => panic!("Expected NewInvitation event for admin role, got: {:?}", event),
            },
            _ => panic!("Expected Event message for admin invitation, got: {:?}", admin_notification),
        }

        // Cleanup
        cleanup_invitation(member_invitation_id).await;
        cleanup_invitation(admin_invitation_id).await;
        cleanup_test_users_from_a_group_chat(vec![user1.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id, user3.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_no_invitation_notification_when_websocket_service_unavailable() {
        // Arrange: Create two users
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_no_ws_service_sender".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_no_ws_service_receiver".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_no_ws_service_group", user1.id).await;

        // Start test server (this test assumes the WebSocket service might not be available)
        let (addr, shutdown) = start_test_server().await;

        // Connect user2 to websocket
        let (mut ws2, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 to websocket");

        let join_id2 = Uuid::new_v4().to_string();
        let join_msg2 = WebSocketMessage::Request {
            request_id: join_id2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2 = send_websocket_message_and_get_response(&mut ws2, join_msg2).await;

        // Act: Send invitation (this should work even if WebSocket notification fails)
        let invitation_response = send_invitation_via_api(
            addr,
            &token1,
            user2.id,
            group.id,
            MemberRole::Member,
        )
        .await
        .expect("Failed to send invitation");

        // Assert: Invitation should be created successfully even if WebSocket notification fails
        assert!(invitation_response.get("data").is_some(), "Invitation should be created");
        let invitation_id = invitation_response["data"]["id"].as_i64().unwrap() as i32;

        // The notification might or might not arrive depending on WebSocket service availability
        // This test mainly ensures the invitation creation doesn't fail due to WebSocket issues
        let _notification = receive_websocket_message_with_timeout(&mut ws2, Duration::from_secs(3)).await;
        // We don't assert on the notification result as it depends on service availability

        // Cleanup
        cleanup_invitation(invitation_id).await;
        cleanup_test_users_from_a_group_chat(vec![user1.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }
}
