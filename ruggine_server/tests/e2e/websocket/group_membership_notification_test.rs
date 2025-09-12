use axum::http::StatusCode;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;
use tokio::time::timeout;

use ruggine_server::websocket::group_message::{GroupAction, GroupEvent};
use ruggine_server::websocket::message::{ClientAction, ServerEvent, WebSocketMessage};
use ruggine_server::dto::invitation_dto::{InvitationCreateDto, InvitationUpdateStatusDto};
use ruggine_server::dto::group_membership_dto::LeaveGroupMembershipDto;
use ruggine_server::entity::group_membership::MemberRole;
use ruggine_server::entity::invitation::InvitationStatus;

use crate::common::{
    cleanup_group_chat, cleanup_test_users, cleanup_test_users_from_a_group_chat,
    create_login_and_get_token, create_test_group_chat_with_invitation_and_membership,
    connect_chat_websocket_with_auth, send_websocket_message_and_get_response, start_test_server,
    add_test_user_to_a_group, cleanup_invitation,
};

#[cfg(test)]
mod group_membership_websocket_notification_e2e_tests {
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

    /// Helper function to accept an invitation via HTTP API
    async fn accept_invitation_via_api(
        addr: std::net::SocketAddr,
        token: &str,
        invitation_id: i32,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let accept_payload = InvitationUpdateStatusDto {
            invitation_id,
            status: InvitationStatus::Accepted,
        };

        let response = client
            .patch(format!("http://{}/api/invitation/update-status", addr))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&accept_payload)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;
        Ok(response_json)
    }

    /// Helper function to leave a group via HTTP API
    async fn leave_group_via_api(
        addr: std::net::SocketAddr,
        token: &str,
        membership_id: i32,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let leave_payload = LeaveGroupMembershipDto {
            id: membership_id,
        };

        let response = client
            .patch(format!("http://{}/api/group_membership/leave", addr))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&leave_payload)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;
        Ok(response_json)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_new_group_membership_notification_when_user_accepts_invitation() {
        // Arrange: Create two users and a group where user1 is the owner
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_new_membership_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_new_membership_user2".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_new_membership_group", user1.id).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user1 to websocket and join the group
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

        // Send an invitation from user1 to user2
        let invitation_response = send_invitation_via_api(addr, &token1, user2.id, group.id, MemberRole::Member)
            .await
            .expect("Failed to send invitation");

        let invitation_id = invitation_response["data"]["id"].as_i64().unwrap() as i32;

        // Act: User2 accepts the invitation, which should trigger a NewGroupMembership notification
        let accept_response = accept_invitation_via_api(addr, &token2, invitation_id)
            .await
            .expect("Failed to accept invitation");

        // Verify the acceptance was successful
        assert_eq!(accept_response["data"]["status"], "accepted");

        // Assert: User1 should receive a NewGroupMembership notification
        let notification = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have a message");

        match notification {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::NewGroupMembership { group_id, new_membership_username }) => {
                        assert_eq!(group_id, group.id, "Should notify about the correct group");
                        assert_eq!(new_membership_username, user2.username, "Should contain the new member's username");
                    }
                    _ => panic!("Expected NewGroupMembership event, got: {:?}", event),
                }
            }
            _ => panic!("Expected Event message, got: {:?}", notification),
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id, user2.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_left_group_membership_notification_when_user_leaves_group() {
        // Arrange: Create two users, both members of the same group
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_left_membership_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_left_membership_user2".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_left_membership_group", user1.id).await;
        
        // Add user2 to the group
        let membership2 = add_test_user_to_a_group(user2.id, &group).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect both users to websocket and join the group
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

        // Act: User2 leaves the group via HTTP API
        let leave_response = leave_group_via_api(addr, &token2, membership2.id)
            .await
            .expect("Failed to leave group");

        // Verify the leave was successful
        assert_eq!(leave_response["data"]["membership_status"], "left");

        // Assert: User1 should receive a LeftGroupMembership notification
        let notification = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have a message");

        match notification {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::LeftGroupMembership { group_id, left_membership_username }) => {
                        assert_eq!(group_id, group.id, "Should notify about the correct group");
                        assert_eq!(left_membership_username, user2.username, "Should contain the leaving member's username");
                    }
                    _ => panic!("Expected LeftGroupMembership event, got: {:?}", event),
                }
            }
            _ => panic!("Expected Event message, got: {:?}", notification),
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id, user2.id], group.id).await; // user2 already left
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_left_group_membership_notification_received_by_leaving_user() {
        // Arrange: Create two users, both members of the same group
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_left_self_notification_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_left_self_notification_user2".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_left_self_notification_group", user1.id).await;
        
        // Add user2 to the group
        let membership2 = add_test_user_to_a_group(user2.id, &group).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user2 to websocket and join the group (the user who will leave)
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

        // Act: User2 leaves the group via HTTP API
        let leave_response = leave_group_via_api(addr, &token2, membership2.id)
            .await
            .expect("Failed to leave group");

        // Verify the leave was successful
        assert_eq!(leave_response["data"]["membership_status"], "left");

        // Assert: User2 should receive a notification about leaving (since they're still connected)
        let notification = receive_websocket_message_with_timeout(&mut ws2, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have a message");

        match notification {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::LeftGroupMembership { group_id, left_membership_username }) => {
                        assert_eq!(group_id, group.id, "Should notify about the correct group");
                        assert_eq!(left_membership_username, user2.username, "Should contain the leaving member's username");
                    }
                    _ => panic!("Expected LeftGroupMembership event, got: {:?}", event),
                }
            }
            _ => panic!("Expected Event message, got: {:?}", notification),
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id, user2.id], group.id).await; // user2 already left
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_new_group_membership_notification_multiple_connected_users() {
        // Arrange: Create three users, user1 and user3 are already in the group
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_multi_new_membership_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_multi_new_membership_user2".to_string()).await;
        let (user3, _password3, token3) =
            create_login_and_get_token("e2e_multi_new_membership_user3".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_multi_new_membership_group", user1.id).await;
        
        // Add user3 to the group
        let _membership3 = add_test_user_to_a_group(user3.id, &group).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user1 and user3 to websocket and join the group
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

        // Clear any pending join notifications
        let _ = receive_websocket_message_with_timeout(&mut ws1, Duration::from_millis(500)).await;
        let _ = receive_websocket_message_with_timeout(&mut ws3, Duration::from_millis(500)).await;

        // Send an invitation from user1 to user2
        let invitation_response = send_invitation_via_api(addr, &token1, user2.id, group.id, MemberRole::Member)
            .await
            .expect("Failed to send invitation");

        let invitation_id = invitation_response["data"]["id"].as_i64().unwrap() as i32;

        // Act: User2 accepts the invitation
        let _accept_response = accept_invitation_via_api(addr, &token2, invitation_id)
            .await
            .expect("Failed to accept invitation");

        // Assert: Both user1 and user3 should receive NewGroupMembership notifications
        let notification1 = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have a message");

        match notification1 {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::NewGroupMembership { group_id, new_membership_username }) => {
                        assert_eq!(group_id, group.id, "Should notify about the correct group");
                        assert_eq!(new_membership_username, user2.username, "Should contain the new member's username");
                    }
                    _ => panic!("Expected NewGroupMembership event for user1, got: {:?}", event),
                }
            }
            _ => panic!("Expected Event message for user1, got: {:?}", notification1),
        }

        let notification3 = receive_websocket_message_with_timeout(&mut ws3, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have a message");

        match notification3 {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::NewGroupMembership { group_id, new_membership_username }) => {
                        assert_eq!(group_id, group.id, "Should notify about the correct group");
                        assert_eq!(new_membership_username, user2.username, "Should contain the new member's username");
                    }
                    _ => panic!("Expected NewGroupMembership event for user3, got: {:?}", event),
                }
            }
            _ => panic!("Expected Event message for user3, got: {:?}", notification3),
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id, user2.id, user3.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id, user3.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_left_group_membership_notification_multiple_connected_users() {
        // Arrange: Create three users, all members of the same group
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_multi_left_membership_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_multi_left_membership_user2".to_string()).await;
        let (user3, _password3, token3) =
            create_login_and_get_token("e2e_multi_left_membership_user3".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_multi_left_membership_group", user1.id).await;
        
        // Add user2 and user3 to the group
        let membership2 = add_test_user_to_a_group(user2.id, &group).await;
        let _membership3 = add_test_user_to_a_group(user3.id, &group).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect all users to websocket and join the group
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

        // Clear any pending join notifications
        let _ = receive_websocket_message_with_timeout(&mut ws1, Duration::from_millis(500)).await;
        let _ = receive_websocket_message_with_timeout(&mut ws1, Duration::from_millis(500)).await; // user2 and user3 joins
        let _ = receive_websocket_message_with_timeout(&mut ws3, Duration::from_millis(500)).await;

        // Act: User2 leaves the group
        let _leave_response = leave_group_via_api(addr, &token2, membership2.id)
            .await
            .expect("Failed to leave group");

        // Assert: User1 and user3 should receive LeftGroupMembership notifications
        let notification1 = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have a message");

        match notification1 {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::LeftGroupMembership { group_id, left_membership_username }) => {
                        assert_eq!(group_id, group.id, "Should notify about the correct group");
                        assert_eq!(left_membership_username, user2.username, "Should contain the leaving member's username");
                    }
                    _ => panic!("Expected LeftGroupMembership event for user1, got: {:?}", event),
                }
            }
            _ => panic!("Expected Event message for user1, got: {:?}", notification1),
        }

        let notification3 = receive_websocket_message_with_timeout(&mut ws3, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have a message");

        match notification3 {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::LeftGroupMembership { group_id, left_membership_username }) => {
                        assert_eq!(group_id, group.id, "Should notify about the correct group");
                        assert_eq!(left_membership_username, user2.username, "Should contain the leaving member's username");
                    }
                    _ => panic!("Expected LeftGroupMembership event for user3, got: {:?}", event),
                }
            }
            _ => panic!("Expected Event message for user3, got: {:?}", notification3),
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id, user2.id, user3.id], group.id).await; // user2 already left
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id, user3.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_no_notification_when_user_not_connected_to_websocket() {
        // Arrange: Create two users where user1 is in group but not connected to websocket
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_no_notif_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_no_notif_user2".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_no_notif_group", user1.id).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Don't connect user1 to websocket - this should test that notifications 
        // are not sent to users who are not connected

        // Send an invitation from user1 to user2
        let invitation_response = send_invitation_via_api(addr, &token1, user2.id, group.id, MemberRole::Member)
            .await
            .expect("Failed to send invitation");

        let invitation_id = invitation_response["data"]["id"].as_i64().unwrap() as i32;

        // Act: User2 accepts the invitation
        let accept_response = accept_invitation_via_api(addr, &token2, invitation_id)
            .await
            .expect("Failed to accept invitation");

        // Verify the acceptance was successful
        assert_eq!(accept_response["data"]["status"], "accepted");

        // Assert: Since user1 is not connected to websocket, no notification should be sent
        // This is primarily testing that the system doesn't crash when trying to send notifications
        // to users who are not connected. The actual verification would be in the logs.

        // Wait a bit to ensure any potential notifications would have been processed
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id, user2.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_new_group_membership_with_admin_role() {
        // Arrange: Create two users where user2 will join as admin
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_admin_membership_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_admin_membership_user2".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_admin_membership_group", user1.id).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user1 to websocket and join the group
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

        // Send an admin invitation from user1 to user2
        let invitation_response = send_invitation_via_api(addr, &token1, user2.id, group.id, MemberRole::Admin)
            .await
            .expect("Failed to send admin invitation");

        let invitation_id = invitation_response["data"]["id"].as_i64().unwrap() as i32;

        // Act: User2 accepts the admin invitation
        let accept_response = accept_invitation_via_api(addr, &token2, invitation_id)
            .await
            .expect("Failed to accept admin invitation");

        // Verify the acceptance was successful
        assert_eq!(accept_response["data"]["status"], "accepted");

        // Assert: User1 should receive a NewGroupMembership notification regardless of role
        let notification = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have a message");

        match notification {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::NewGroupMembership { group_id, new_membership_username }) => {
                        assert_eq!(group_id, group.id, "Should notify about the correct group");
                        assert_eq!(new_membership_username, user2.username, "Should contain the new admin member's username");
                    }
                    _ => panic!("Expected NewGroupMembership event, got: {:?}", event),
                }
            }
            _ => panic!("Expected Event message, got: {:?}", notification),
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id, user2.id], group.id).await;
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_left_group_membership_notification_with_multiple_user_connections() {
        // Arrange: Create two users where user2 has multiple websocket connections
        let (user1, _password1, token1) =
            create_login_and_get_token("e2e_multi_conn_left_user1".to_string()).await;
        let (user2, _password2, token2) =
            create_login_and_get_token("e2e_multi_conn_left_user2".to_string()).await;

        // Create a group owned by user1
        let group = create_test_group_chat_with_invitation_and_membership("e2e_multi_conn_left_group", user1.id).await;
        
        // Add user2 to the group
        let membership2 = add_test_user_to_a_group(user2.id, &group).await;

        // Start test server
        let (addr, shutdown) = start_test_server().await;

        // Connect user1 to websocket and join the group
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

        // Connect user2 with first connection
        let (mut ws2_conn1, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 first connection to websocket");

        let join_id2_1 = Uuid::new_v4().to_string();
        let join_msg2_1 = WebSocketMessage::Request {
            request_id: join_id2_1.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2_1 = send_websocket_message_and_get_response(&mut ws2_conn1, join_msg2_1)
            .await
            .expect("User2 first connection failed to join group");

        // Connect user2 with second connection  
        let (mut ws2_conn2, _) = connect_chat_websocket_with_auth(addr, &token2)
            .await
            .expect("Failed to connect user2 second connection to websocket");

        let join_id2_2 = Uuid::new_v4().to_string();
        let join_msg2_2 = WebSocketMessage::Request {
            request_id: join_id2_2.clone(),
            action: ClientAction::Groups(GroupAction::Join {}),
        };
        let _response2_2 = send_websocket_message_and_get_response(&mut ws2_conn2, join_msg2_2)
            .await
            .expect("User2 second connection failed to join group");

        // Clear any pending join notifications
        let _ = receive_websocket_message_with_timeout(&mut ws1, Duration::from_millis(500)).await;

        // Act: User2 leaves the group via HTTP API
        let leave_response = leave_group_via_api(addr, &token2, membership2.id)
            .await
            .expect("Failed to leave group");

        // Verify the leave was successful
        assert_eq!(leave_response["data"]["membership_status"], "left");

        // Assert: User1 should receive a LeftGroupMembership notification
        let notification1 = receive_websocket_message_with_timeout(&mut ws1, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have a message");

        match notification1 {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::LeftGroupMembership { group_id, left_membership_username }) => {
                        assert_eq!(group_id, group.id, "Should notify about the correct group");
                        assert_eq!(left_membership_username, user2.username, "Should contain the leaving member's username");
                    }
                    _ => panic!("Expected LeftGroupMembership event for user1, got: {:?}", event),
                }
            }
            _ => panic!("Expected Event message for user1, got: {:?}", notification1),
        }

        // Assert: Both of user2's connections should receive the notification
        let notification2_conn1 = receive_websocket_message_with_timeout(&mut ws2_conn1, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have a message");

        match notification2_conn1 {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::LeftGroupMembership { group_id, left_membership_username }) => {
                        assert_eq!(group_id, group.id, "Should notify about the correct group");
                        assert_eq!(left_membership_username, user2.username, "Should contain the leaving member's username");
                    }
                    _ => panic!("Expected LeftGroupMembership event for user2 conn1, got: {:?}", event),
                }
            }
            _ => panic!("Expected Event message for user2 conn1, got: {:?}", notification2_conn1),
        }

        let notification2_conn2 = receive_websocket_message_with_timeout(&mut ws2_conn2, Duration::from_secs(5))
            .await
            .expect("Should receive notification")
            .expect("Should have a message");

        match notification2_conn2 {
            WebSocketMessage::Event { event, .. } => {
                match event {
                    ServerEvent::Groups(GroupEvent::LeftGroupMembership { group_id, left_membership_username }) => {
                        assert_eq!(group_id, group.id, "Should notify about the correct group");
                        assert_eq!(left_membership_username, user2.username, "Should contain the leaving member's username");
                    }
                    _ => panic!("Expected LeftGroupMembership event for user2 conn2, got: {:?}", event),
                }
            }
            _ => panic!("Expected Event message for user2 conn2, got: {:?}", notification2_conn2),
        }

        // Cleanup
        cleanup_test_users_from_a_group_chat(vec![user1.id, user2.id], group.id).await; // user2 already left
        cleanup_group_chat(group.id).await;
        cleanup_test_users(vec![user1.id, user2.id]).await;
        shutdown.send(()).unwrap();
    }
}