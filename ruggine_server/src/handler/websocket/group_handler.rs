use crate::dto::websocket::WebSocketQuery;
use crate::error::connection_error::ConnectionError;
use crate::error::request_error::ValidatedWebSocketMessage;
use crate::error::web_socket_error::WebSocketError;
use crate::service::websocket::WebSocketGroupService;
use crate::state::websocket::WebSocketState;
use crate::websocket::group_message::GroupAction::{Join, Leave};
use crate::websocket::group_message::GroupEvent::NewMessage;
use crate::websocket::message::{ControlMessage, ServerEvent, WsError};
use crate::websocket::{ClientAction, WebSocketConnection, WebSocketManager, WebSocketMessage};
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::Response;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use axum::Extension;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use crate::entity::user::User;

pub async fn group_websocket_handler(
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
        handle_group_websocket_upgrade(
            socket,
            user_id,
            state.manager.clone(),
            state.group_service.clone(),
        )
    }))
}

async fn handle_group_websocket_upgrade(
    socket: WebSocket,
    user_id: i32,
    manager: Arc<WebSocketManager>,
    group_service: Arc<WebSocketGroupService>,
) {
    info!(
        "WebSocket group connection established for user {}",
        user_id
    );
    handle_group_websocket_connection(socket, user_id, manager, group_service).await;
    info!("WebSocket group connection closed for user {}", user_id);
}

pub async fn handle_group_websocket_connection(
    socket: WebSocket,
    user_id: i32,
    manager: Arc<WebSocketManager>,
    group_service: Arc<WebSocketGroupService>,
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

                    match handle_group_client_message(
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

    // Cleanup
    group_service.cleanup_connection(&connection_id).await;
    info!("Cleaned up group WebSocket connection for user {}", user_id);
}

async fn handle_group_client_message(
    user_id: i32,
    connection_id: &str,
    message: WebSocketMessage,
    manager: &Arc<WebSocketManager>,
    group_service: &Arc<WebSocketGroupService>,
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
                }
                Leave {} => {
                    info!(
                        "User {} requesting to leave group via connection {}",
                        user_id, connection_id
                    );
                    group_service
                        .unsubscribe(user_id)
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

// nel tuo handler "nuovo messaggio gruppo"
pub async fn handle_new_group_message(
    group_service: Arc<WebSocketGroupService>,
    manager: Arc<WebSocketManager>,
    group_id: i32,
    message: WebSocketMessage,
) {
    // chiediamo al service chi sono gli utenti
    let connection_ids = match group_service.broadcast_to_group(group_id).await {
        Ok(res) => res,
        Err(e) => {
            warn!("Errore broadcast group {}: {:?}", group_id, e);
            return;
        }
    };

    for conn_id in connection_ids {
        if let Err(e) = manager.send_to_connection(&conn_id, message.clone()).await {
            warn!("Failed to send to connection {}: {}", conn_id, e);
        }
    }
}
