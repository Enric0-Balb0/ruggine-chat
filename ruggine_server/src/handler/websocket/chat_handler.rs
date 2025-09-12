use crate::error::connection_error::ConnectionError;
use crate::error::request_error::ValidatedWebSocketMessage;
use crate::error::web_socket_error::WebSocketError;
use crate::service::websocket::{WebSocketGroupService, WebSocketGroupServiceTrait};
use crate::state::websocket::WebSocketState;
use crate::websocket::message::WebSocketQuery;
use crate::websocket::group_message::GroupAction::{Join, Leave};
use crate::websocket::group_message::GroupEvent::NewMessage;
use crate::websocket::message::{ControlMessage, ServerEvent, WsError};
use crate::websocket::{ClientAction, GroupEvent, WebSocketConnection, WebSocketManager, WebSocketMessage};
use crate::websocket::core::manager_trait::WebSocketManagerTrait;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::Response;
use futures::{SinkExt, StreamExt};
use utoipa::openapi::info;
use std::sync::Arc;
use axum::Extension;
use chrono::Utc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use crate::dto::text_message_dto::{TextMessageInfoReadDto, TextMessageReadDto};
use crate::dto::user_dto::UpdateOnlineDto;
use crate::entity::user::User;

pub async fn chat_websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<WebSocketState>,
    Query(params): Query<WebSocketQuery>,
    Extension(user): Extension<User>,
) -> Result<Response, StatusCode> {
    let user_id = user.id;
    info!(
        "WebSocket group connection attempt for user {} with valid token",
        user_id
    );

    let arc_state = Arc::new(state);

    Ok(ws.on_upgrade(move |socket| {
        handle_chat_websocket_upgrade(
            socket,
            user_id,
            Arc::clone(&arc_state),
        )
    }))
}

async fn handle_chat_websocket_upgrade(
    socket: WebSocket,
    user_id: i32,
    state: Arc<WebSocketState>,
) {
    info!(
        "WebSocket group connection established for user {}",
        user_id
    );
    handle_chat_websocket_connection(socket, user_id, state).await;
    info!("WebSocket group connection closed for user {}", user_id);
}

pub async fn handle_chat_websocket_connection(
    socket: WebSocket,
    user_id: i32,
    arc_state: Arc<WebSocketState>,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WebSocketMessage>();

    // Task per inviare messaggi
    let send_task = tokio::spawn({
        let user_id = user_id;
        async move {
            while let Some(message) = rx.recv().await {
                if let Ok(json) = message.to_json() {
                    if let Err(e) = ws_sender.send(Message::Text(json)).await {
                        warn!("Failed to send message to user {}: {}", user_id, e);
                        break;
                    }
                } else {
                    error!("Failed to serialize message for user {}", user_id);
                }
            }
        }
    });

    // Registrazione della connessione PRIMA di spawnare il task di ricezione
    let connection = arc_state.manager.register_connection(user_id, tx.clone()).await;
    let connection_id = connection.connection_id.clone();

    // Task per ricevere messaggi
    let receive_task = {
        let arc_state = Arc::clone(&arc_state);
        let user_id = user_id;
        let tx = tx.clone();
        let connection_id = connection_id.clone();

        tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                if let Ok(Message::Text(text)) = msg {
                    let validated =
                        match ValidatedWebSocketMessage::from_text_or_error(&text, &tx).await {
                            Ok(m) => m,
                            Err(_) => break,
                        };

                    match handle_chat_client_message(
                        user_id,
                        &connection_id,
                        validated.0,
                        Arc::clone(&arc_state),
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(_) => {
                            break;
                        }
                    }
                }
            }
        })
    };

    WebSocketConnection::await_connection_tasks(send_task, connection, receive_task).await;

    // Handle connection closed or user left
    arc_state
        .group_service
        .unsubscribe(user_id, &*connection_id)
        .await;

    info!(
        "User {} successfully left group via connection {}",
        user_id, connection_id
    );

    match mark_user_offline_and_signal_user_left(user_id, &arc_state).await {
        Ok(_) => {},
        Err(e) => error!("Something went wrong signaling users of {} left: {:?}", user_id, e),
    }

    // Cleanup
    arc_state.group_service.unsubscribe(user_id, &connection_id).await;
    info!("Cleaned up group WebSocket connection for user {}", user_id);
}

pub async fn handle_chat_client_message(
    user_id: i32,
    connection_id: &str,
    message: WebSocketMessage,
    arc_state: Arc<WebSocketState>,
) -> Result<(), WsError> {
    match message {
        WebSocketMessage::Request { request_id, action } => match action {
            ClientAction::Groups(group_action) => match group_action {
                Join{} => {
                    info!(
                        "User {} requesting to join group via connection {}",
                        user_id, connection_id
                    );
                    arc_state
                        .group_service
                        .subscribe(user_id, connection_id)
                        .await;

                    mark_user_online_and_signal_user_joined(user_id, &arc_state).await?;

                    let message = WebSocketMessage::Response {
                        request_id,
                        ok: true,
                        data: None,
                        error: None,
                    };

                    arc_state
                        .manager
                        .send_to_connection(connection_id, message)
                        .await
                        .map_err(|e| <ConnectionError as Into<WsError>>::into(e))?;

                    info!(
                        "User {} successfully joined group via connection {}",
                        user_id, connection_id
                    );
                }
                Leave {} => {
                    info!(
                        "User {} requesting to leave group via connection {}",
                        user_id, connection_id
                    );


                    let message = WebSocketMessage::Response {
                        request_id,
                        ok: true,
                        data: None,
                        error: None,
                    };
                    arc_state
                        .manager
                        .send_to_connection(connection_id, message)
                        .await
                        .map_err(|e| <ConnectionError as Into<WsError>>::into(e))?;
                    
                    return Err(WsError {
                        code: 0,
                        message: "Leave group".into(),
                    });
                }
            },
            _ => warn!("Unhandled message type from user {}: {:?}", user_id, action),
        },
        WebSocketMessage::Control(control_message) => match control_message {
            ControlMessage::Ping => {
                debug!("Ping received from user {}", user_id);
                let message = WebSocketMessage::Control(ControlMessage::Pong);
                arc_state
                    .manager
                    .send_to_connection(connection_id, message)
                    .await
                    .map_err(|e| <ConnectionError as Into<WsError>>::into(e))?;
            }
            _ => warn!(
                "Unhandled message type from user {}: {:?}",
                user_id, control_message
            ),
        },
        _ => warn!(
            "Unhandled message type from user {}: {:?}",
            user_id, message
        ),
    }
    Ok(())
}

async fn mark_user_online_and_signal_user_joined(user_id: i32, state: &Arc<WebSocketState>) -> Result<(), WsError> {
    // Don't signal if user have more than 1 connection
    if state.group_service.get_connections_number_for_user_id(user_id).await > 1 {
        return Ok(());
    }

    // Mark user online
    match state
        .user_service
        .update_online(user_id, UpdateOnlineDto {online: true})
        .await {
        Ok(_) => (),
        Err(e) => {
            error!("Failed to mark user {} online: {:?}", user_id ,e);
            return Err(WsError {
                code: 0,
                message: format!("Failed to mark user {} online ", user_id),
            });
        },
    }

    // Now signal all users connected to user_id that he is online
    let connection_ids = state
        .group_service
        .connections_to_broadcast_new_user_joined(user_id)
        .await
        .map_err(|e| <WebSocketError as Into<WsError>>::into(e))?;

    let notification = WebSocketMessage::Event {
        event: ServerEvent::Groups(GroupEvent::Joined {
            user_id
        }),
        timestamp: Utc::now(),
    };

    for conn_id in connection_ids {
        if let Err(e) = state.manager.send_to_connection(&conn_id.1, notification.clone()).await {
            warn!("Failed to send joined message to connection {}: {}", conn_id.1, e);
            continue;
        }
    }
    Ok(())
}

async fn mark_user_offline_and_signal_user_left(user_id: i32, state: &Arc<WebSocketState>) -> Result<(), WsError> {
    // Don't signal if user still have some connections
    let connections_number = state.group_service.get_connections_number_for_user_id(user_id).await;
    if connections_number > 0 {
        info!("User {} still have {} connections opened", user_id, connections_number);
        return Ok(());
    }

    // Mark user offline
    match state
        .user_service
        .update_online(user_id, UpdateOnlineDto {online: false})
        .await {
        Ok(_) => {
            info!("User {} marked offline in the db", user_id);
        },
        Err(e) => {
            error!("Failed to mark user {} offline: {:?}", user_id, e);
            return Err(WsError {
                code: 0,
                message: format!("Failed to mark user {} offline", user_id),
            });
        },
    }

    // Now signal all users connected to user_id that he is offline
    let connection_ids = state
        .group_service
        .connections_to_broadcast_new_user_joined(user_id)
        .await
        .map_err(|e| <WebSocketError as Into<WsError>>::into(e))?;

    let notification = WebSocketMessage::Event {
        event: ServerEvent::Groups(GroupEvent::Left {
            user_id
        }),
        timestamp: Utc::now(),
    };

    for conn_id in connection_ids {
        if let Err(e) = state.manager.send_to_connection(&conn_id.1, notification.clone()).await {
            warn!("Failed to send left message to connection {}: {}", conn_id.1, e);
            continue;
        }
    }
    info!("Signaled all users connected that user {} left group ws", user_id);
    Ok(())
}

// Handler for receiving new message from the server
pub async fn handle_new_group_message(
    group_service: Arc<dyn WebSocketGroupServiceTrait>,
    manager: Arc<dyn WebSocketManagerTrait>,
    group_id: i32,
    text_message: TextMessageReadDto,
    sender_user_username: String,
) {
    // Search for active connections in the group
    let connection_ids = match group_service.connections_to_broadcast_by_group_id(group_id).await {
        Ok(res) => res,
        Err(e) => {
            warn!("Errore broadcast group {}: {:?}", group_id, e);
            return;
        }
    };

    let notification = WebSocketMessage::Event {
        event: ServerEvent::Groups(GroupEvent::NewMessage {
            message_id: text_message.id,
            group_id,
            sender_id: text_message.sender_id,
            sender_username: sender_user_username.clone(),
            content: text_message.content.clone(),
            sent_at: text_message.sent_at,
        }),
        timestamp: Utc::now(),
    };

    for conn_id in connection_ids {
        group_service.update_sent_at_for_a_user(conn_id.0, text_message.id).await;
        if let Err(e) = manager.send_to_connection(&conn_id.1, notification.clone()).await {
            warn!("Failed to send to connection {}: {}", conn_id.1, e);
            continue;
        }
    }
}

pub async fn handle_new_read_text_message(
    group_service: Arc<dyn WebSocketGroupServiceTrait>,
    manager: Arc<dyn WebSocketManagerTrait>,
    text_message_info: TextMessageInfoReadDto,
    group_id: i32,
) {
    // Search for active connections for the user
    let connection_ids = match group_service.connections_to_broadcast_by_user_id(text_message_info.user_id).await {
        Ok(res) => res,
        Err(e) => {
            warn!("Errore broadcast new_read_text_message user {}: {:?}", text_message_info.user_id, e);
            return;
        }
    };

    let notification = WebSocketMessage::Event {
        event: ServerEvent::Groups(GroupEvent::NewReadTextMessage {
            group_id,
            text_message_id: text_message_info.text_message_id,
            user_id: text_message_info.user_id,
        }),
        timestamp: Utc::now(),
    };

    info!("Broadcasting NewReadTextMessage for message {} in group {} to {} connections", text_message_info.text_message_id, group_id, connection_ids.len());

    for conn_id in connection_ids {
        if let Err(e) = manager.send_to_connection(&conn_id, notification.clone()).await {
            warn!("Failed to send to connection {}: {}", conn_id, e);
            continue;
        }
    }

    info!("Broadcasted NewReadTextMessage for message {} in group {} to all connections", text_message_info.text_message_id, group_id);
}

pub async fn handle_new_invitation(
    group_service: Arc<dyn WebSocketGroupServiceTrait>,
    manager: Arc<dyn WebSocketManagerTrait>,
    invitation_id: i32,
    to_user_id: i32,
) {
    // Search for active connections in the group
    let connection_ids = match group_service.connections_to_broadcast_by_user_id(to_user_id).await {
        Ok(res) => res,
        Err(e) => {
            warn!("Error finding connections for new invitation id {} for user id {}: {:?}", invitation_id, to_user_id, e);
            return;
        }
    };

    let notification = WebSocketMessage::Event {
        event: ServerEvent::Groups(GroupEvent::NewInvitation {
            invitation_id,
        }),
        timestamp: Utc::now(),
    };

    info!("Broadcasting new invitation {} to {} connections", invitation_id, connection_ids.len());

    for conn_id in connection_ids {
        if let Err(e) = manager.send_to_connection(&conn_id, notification.clone()).await {
            warn!("Failed to send to connection {}: {}", conn_id, e);
            continue;
        }
    }

    info!("Broadcasted new invitation {} to all connections", invitation_id);
}

pub async fn handle_new_group_chat(
    group_service: Arc<dyn WebSocketGroupServiceTrait>,
    manager: Arc<dyn WebSocketManagerTrait>,
    group_chat_id: i32,
    created_by_user_id: i32,
) {
    // Search for active connections in the group
    let connection_ids = match group_service.connections_to_broadcast_by_user_id(created_by_user_id).await {
        Ok(res) => res,
        Err(e) => {
            warn!("Error finding connections for new group chat id {} for user id {}: {:?}", group_chat_id, created_by_user_id, e);
            return;
        }
    };

    let notification = WebSocketMessage::Event {
        event: ServerEvent::Groups(GroupEvent::NewGroupChat {
            group_chat_id,
        }),
        timestamp: Utc::now(),
    };

    info!("Broadcasting new group chat {} to {} connections", group_chat_id, connection_ids.len());

    for conn_id in connection_ids {
        if let Err(e) = manager.send_to_connection(&conn_id, notification.clone()).await {
            warn!("Failed to send to connection {}: {}", conn_id, e);
            continue;
        }
    }

    info!("Broadcasted new group chat {} to all connections", group_chat_id);
}

pub async fn handle_new_group_membership(
    group_service: Arc<dyn WebSocketGroupServiceTrait>,
    manager: Arc<dyn WebSocketManagerTrait>,
    group_id: i32,
    new_membership_username: String,
) {
    // Search for active connections in the group
    let connection_ids = match group_service.connections_to_broadcast_by_group_id(group_id).await {
        Ok(res) => res,
        Err(e) => {
            warn!("Errore broadcast group {}: {:?}", group_id, e);
            return;
        }
    };

    let notification = WebSocketMessage::Event {
        event: ServerEvent::Groups(GroupEvent::NewGroupMembership {
            group_id,
            new_membership_username,
        }),
        timestamp: Utc::now(),
    };

    info!("Broadcasting new membership in group {} to {} connections", group_id, connection_ids.len());

    for conn_id in connection_ids {
        if let Err(e) = manager.send_to_connection(&conn_id.1, notification.clone()).await {
            warn!("Failed to send to connection {}: {}", conn_id.1, e);
            continue;
        }
    }

    info!("Broadcasted new membership in group {} to all connections", group_id);
}

pub async fn handle_left_group_membership(
    group_service: Arc<dyn WebSocketGroupServiceTrait>,
    manager: Arc<dyn WebSocketManagerTrait>,
    group_id: i32,
    left_user_id: i32,
    left_membership_username: String,
) {
    // Search for active connections in the group
    let mut connection_ids = match group_service.connections_to_broadcast_by_group_id(group_id).await {
        Ok(res) => res.into_iter().map(|conn| conn.1).collect::<Vec<String>>(),
        Err(e) => {
            warn!("Errore broadcast group {}: {:?}", group_id, e);
            return;
        }
    };

    // Also search for active connections of the user who left
    let user_connection_ids = match group_service.connections_to_broadcast_by_user_id(left_user_id).await {
        Ok(res) => res,
        Err(e) => {
            warn!("Error finding connections for left user id {}: {:?}", left_user_id, e);
            Vec::new() // Continue with empty vec if error
        }
    };

    // Merge the connection lists (avoid duplicates)
    for user_conn_id in user_connection_ids {
        if !connection_ids.contains(&user_conn_id) {
            connection_ids.push(user_conn_id);
        }
    }

    let notification = WebSocketMessage::Event {
        event: ServerEvent::Groups(GroupEvent::LeftGroupMembership {
            group_id,
            left_membership_username,
        }),
        timestamp: Utc::now(),
    };

    info!("Broadcasting left membership in group {} to {} connections", group_id, connection_ids.len());

    for conn_id in connection_ids {
        if let Err(e) = manager.send_to_connection(&conn_id, notification.clone()).await {
            warn!("Failed to send to connection {}: {}", conn_id, e);
            continue;
        }
    }

    info!("Broadcasted left membership in group {} to all connections", group_id);
}