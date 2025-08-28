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
use std::sync::Arc;
use axum::Extension;
use chrono::Utc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use crate::dto::text_message_dto::TextMessageReadDto;
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

    Ok(ws.on_upgrade(move |socket| {
        handle_chat_websocket_upgrade(
            socket,
            user_id,
            state.manager.clone(),
            state.group_service.clone(),
        )
    }))
}

async fn handle_chat_websocket_upgrade(
    socket: WebSocket,
    user_id: i32,
    manager: Arc<dyn WebSocketManagerTrait>,
    group_service: Arc<dyn WebSocketGroupServiceTrait>,
) {
    info!(
        "WebSocket group connection established for user {}",
        user_id
    );
    handle_chat_websocket_connection(socket, user_id, manager, group_service).await;
    info!("WebSocket group connection closed for user {}", user_id);
}

pub async fn handle_chat_websocket_connection(
    socket: WebSocket,
    user_id: i32,
    manager: Arc<dyn WebSocketManagerTrait>,
    group_service: Arc<dyn WebSocketGroupServiceTrait>,
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
    let connection = manager.register_connection(user_id, tx.clone()).await;
    let connection_id = connection.connection_id.clone();

    // Task per ricevere messaggi
    let receive_task = {
        let manager = manager.clone();
        let group_service = group_service.clone();
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
                        &manager,
                        &group_service,
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

    match signal_user_left(user_id, &manager, &group_service).await {
        Ok(_) => {},
        Err(e) => error!("Something went wrong signaling users of {} left: {:?}", user_id, e),
    }

    // Cleanup
    group_service.cleanup_connection(&connection_id).await;
    info!("Cleaned up group WebSocket connection for user {}", user_id);
}

pub async fn handle_chat_client_message(
    user_id: i32,
    connection_id: &str,
    message: WebSocketMessage,
    manager: &Arc<dyn WebSocketManagerTrait>,
    group_service: &Arc<dyn WebSocketGroupServiceTrait>,
) -> Result<(), WsError> {
    match message {
        WebSocketMessage::Request { request_id, action } => match action {
            ClientAction::Groups(group_action) => match group_action {
                Join{} => {
                    info!(
                        "User {} requesting to join group via connection {}",
                        user_id, connection_id
                    );
                    group_service
                        .subscribe(user_id, connection_id)
                        .await
                        .map_err(|e| <WebSocketError as Into<WsError>>::into(e))?;
                    let message = WebSocketMessage::Response {
                        request_id,
                        ok: true,
                        data: None,
                        error: None,
                    };
                    manager
                        .send_to_connection(connection_id, message)
                        .await
                        .map_err(|e| <ConnectionError as Into<WsError>>::into(e))?;

                    signal_user_joined(user_id, manager, group_service).await?;
                }
                Leave {} => {
                    info!(
                        "User {} requesting to leave group via connection {}",
                        user_id, connection_id
                    );
                    group_service
                        .unsubscribe(user_id, connection_id)
                        .await
                        .map_err(|e| <WebSocketError as Into<WsError>>::into(e))?;
                    let message = WebSocketMessage::Response {
                        request_id,
                        ok: true,
                        data: None,
                        error: None,
                    };
                    manager
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
                manager
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

async fn signal_user_joined(user_id: i32, manager: &Arc<dyn WebSocketManagerTrait>, group_service: &Arc<dyn WebSocketGroupServiceTrait>) -> Result<(), WsError> {
    // Don't signal if user have more than 1 connection
    if group_service.get_connections_number_for_user_id(user_id).await > 1 {
        return Ok(());
    }

    // Now signal all users connected to user_id that he is online
    let connection_ids = group_service
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
        if let Err(e) = manager.send_to_connection(&conn_id.1, notification.clone()).await {
            warn!("Failed to send joined message to connection {}: {}", conn_id.1, e);
            continue;
        }
    }
    Ok(())
}

async fn signal_user_left(user_id: i32, manager: &Arc<dyn WebSocketManagerTrait>, group_service: &Arc<dyn WebSocketGroupServiceTrait>) -> Result<(), WsError> {
    // Don't signal if user still have some connections
    if group_service.get_connections_number_for_user_id(user_id).await > 0 {
        return Ok(());
    }

    // Now signal all users connected to user_id that he is offline
    let connection_ids = group_service
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
        if let Err(e) = manager.send_to_connection(&conn_id.1, notification.clone()).await {
            warn!("Failed to send left message to connection {}: {}", conn_id.1, e);
            continue;
        }
    }
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
    /* match message.unwrap() {
        tungstenite::Message::Text(text) => {
            let parsed: WebSocketMessage = serde_json::from_str(&text).unwrap();
            match parsed {
                WebSocketMessage::Event { event, timestamp } => {
                    // Success
                    match event {
                        crate::websocket::ServerEvent::Groups(NewMessage {
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
    } */
    // Search for active connections in the group
    let connection_ids = match group_service.connections_to_broadcast_new_message(group_id).await {
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
        if let Err(e) = manager.send_to_connection(&conn_id.1, notification.clone()).await {
            warn!("Failed to send to connection {}: {}", conn_id.1, e);
            continue;
        }
        group_service.update_sent_at_for_a_user(conn_id.0, text_message.id).await;
    }
}
