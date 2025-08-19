use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use axum::extract::ws::{WebSocket, Message};
use futures::{SinkExt, StreamExt};
use crate::websocket::message::{WebSocketMessage, ClientMessage};
use tracing::{info, error, warn, debug};

#[derive(Debug)]
pub struct WebSocketConnection {
    pub user_id: i32,
    pub connection_id: String,
    pub sender: mpsc::UnboundedSender<WebSocketMessage>,
    is_active: Arc<AtomicBool>,
}

impl WebSocketConnection {
    pub fn new(
        user_id: i32, 
        connection_id: String, 
        sender: mpsc::UnboundedSender<WebSocketMessage>
    ) -> Self {
        Self {
            user_id,
            connection_id,
            sender,
            is_active: Arc::new(AtomicBool::new(true)),
        }
    }
    
    pub async fn send_message(&self, message: WebSocketMessage) -> Result<(), String> {
        if !self.is_active() {
            return Err("Connection is not active".to_string());
        }
        
        self.sender.send(message)
            .map_err(|e| format!("Failed to send message: {}", e))
    }
    
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }
    
    pub fn close(&self) {
        self.is_active.store(false, Ordering::Relaxed);
        debug!("Connection {} marked as closed", self.connection_id);
    }
}

pub async fn handle_websocket_connection(
    socket: WebSocket,
    user_id: i32,
    connection_manager: Arc<crate::websocket::core::manager::WebSocketManager>,
) {
    let connection_id = uuid::Uuid::new_v4().to_string();
    info!("New WebSocket connection for user {} with id {}", user_id, connection_id);
    
    // Crea il canale per i messaggi
    let (message_sender, mut message_receiver) = mpsc::unbounded_channel::<WebSocketMessage>();
    
    // Registra la connessione nel manager
    let connection = connection_manager.register_connection(user_id, message_sender);
    
    let (mut sender, mut receiver) = socket.split();
    
    // Task per inviare messaggi al client
    let send_connection_id = connection_id.clone();
    let send_manager = connection_manager.clone();
    let send_task = tokio::spawn(async move {
        while let Some(message) = message_receiver.recv().await {
            let text = match message.to_json() {
                Ok(json) => json,
                Err(e) => {
                    error!("Failed to serialize message: {}", e);
                    continue;
                }
            };
            
            if sender.send(Message::Text(text)).await.is_err() {
                warn!("Failed to send message to client, connection likely closed");
                break;
            }
        }
        
        // Rimuovi la connessione quando il task termina
        send_manager.unregister_connection(&send_connection_id);
    });
    
    // Task per ricevere messaggi dal client
    let recv_connection_id = connection_id.clone();
    let recv_manager = connection_manager.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    match WebSocketMessage::from_json(&text) {
                        Ok(ws_message) => {
                            handle_client_message(user_id, ws_message, &recv_manager).await;
                        }
                        Err(e) => {
                            error!("Failed to parse WebSocket message: {}", e);
                            let error_msg = WebSocketMessage::error("Invalid message format");
                            let _ = recv_manager.send_to_user(user_id, error_msg).await;
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed for user {}", user_id);
                    break;
                }
                Ok(Message::Ping(_)) => {
                    debug!("Received ping from user {}", user_id);
                    // Il framework gestisce automaticamente il pong
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong from user {}", user_id);
                }
                Ok(Message::Binary(_)) => {
                    warn!("Received binary message, not supported");
                }
                Err(e) => {
                    error!("WebSocket error for user {}: {}", user_id, e);
                    break;
                }
            }
        }

        recv_manager.unregister_connection(&recv_connection_id);
    });
    
    // Aspetta che uno dei task termini
    tokio::select! {
        _ = send_task => {
            info!("Send task completed for user {}", user_id);
        }
        _ = recv_task => {
            info!("Receive task completed for user {}", user_id);
        }
    }
    
    // Assicurati che la connessione sia rimossa
    connection_manager.unregister_connection(&connection_id);
    info!("WebSocket connection cleanup completed for user {}", user_id);
}

async fn handle_client_message(
    user_id: i32,
    message: WebSocketMessage,
    manager: &crate::websocket::core::manager::WebSocketManager,
) {
    match message {
        WebSocketMessage::Connect => {
            info!("User {} connected via WebSocket", user_id);
            
            // Invia conferma di connessione
            let ack = WebSocketMessage::Ack { message_id: None };
            let _ = manager.send_to_user(user_id, ack).await;
        }
        
        WebSocketMessage::Test { message } => {
            info!("Received test message from user {}: {}", user_id, message);
            
            // Risponde con un messaggio di test
            let response = WebSocketMessage::Test {
                message: format!("Echo: {}", message),
            };
            
            let _ = manager.send_to_user(user_id, response).await;
        }
        
        WebSocketMessage::Ping => {
            debug!("Received ping from user {}", user_id);
            
            let pong = WebSocketMessage::Pong;
            let _ = manager.send_to_user(user_id, pong).await;
        }
        
        WebSocketMessage::Disconnect => {
            info!("User {} requested disconnect", user_id);
            // Il cleanup sarà gestito automaticamente quando la connessione si chiude
        }
        
        _ => {
            warn!("Unhandled message type from user {}: {:?}", user_id, message);
        }
    }
}

/// Gestisce una connessione WebSocket specificatamente per un gruppo
pub async fn handle_group_websocket_connection(
    socket: WebSocket,
    user_id: i32,
    group_id: i32,
    manager: Arc<crate::websocket::WebSocketManager>,
    group_service: Arc<crate::service::websocket::WebSocketGroupService>,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<WebSocketMessage>();

    // Registra la connessione
    let connection = manager.register_connection(user_id, tx);
    let connection_id = connection.connection_id.clone();

    // Prova a sottoscrivere l'utente al gruppo
    if let Err(e) = group_service.subscribe_to_group(&connection_id, user_id, group_id).await {
        error!("Failed to subscribe user {} to group {}: {}", user_id, group_id, e);
        // Invia errore e chiudi la connessione
        let error_msg = WebSocketMessage::Error { message: e };
        if let Ok(json) = error_msg.to_json() {
            let _ = ws_sender.send(Message::Text(json)).await;
        }
        return;
    }

    // Task per inviare messaggi dal server al client
    let send_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            match message.to_json() {
                Ok(json) => {
                    if let Err(e) = ws_sender.send(Message::Text(json)).await {
                        warn!("Failed to send message to user {}: {}", user_id, e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to serialize message: {}", e);
                }
            }
        }
    });

    // Task per ricevere messaggi dal client
    let receive_task = tokio::spawn({
        let manager = manager.clone();
        let group_service = group_service.clone();
        let connection_id = connection_id.clone();
        
        async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match WebSocketMessage::from_json(&text) {
                            Ok(message) => {
                                handle_group_client_message(
                                    user_id,
                                    group_id,
                                    &connection_id,
                                    message,
                                    &manager,
                                    &group_service,
                                ).await;
                            }
                            Err(e) => {
                                warn!("Failed to parse message from user {}: {}", user_id, e);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("User {} closed WebSocket connection", user_id);
                        break;
                    }
                    Err(e) => {
                        warn!("WebSocket error for user {}: {}", user_id, e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    });

    // Aspetta che uno dei task finisca
    tokio::select! {
        _ = send_task => {
            debug!("Send task completed for user {}", user_id);
        }
        _ = receive_task => {
            debug!("Receive task completed for user {}", user_id);
        }
    }

    // Cleanup: rimuovi la connessione e le sottoscrizioni
    manager.unregister_connection(&connection_id);
    group_service.cleanup_connection(&connection_id);
    
    info!("Cleaned up group WebSocket connection for user {} in group {}", user_id, group_id);
}

/// Gestisce i messaggi ricevuti dal client in una connessione di gruppo
async fn handle_group_client_message(
    user_id: i32,
    group_id: i32,
    connection_id: &str,
    message: WebSocketMessage,
    manager: &Arc<crate::websocket::WebSocketManager>,
    group_service: &Arc<crate::service::websocket::WebSocketGroupService>,
) {
    match message {
        WebSocketMessage::JoinGroup { group_id: requested_group_id } => {
            // Permetti solo di unirsi al gruppo specifico di questa connessione
            if requested_group_id == group_id {
                if let Err(e) = group_service.subscribe_to_group(connection_id, user_id, group_id).await {
                    warn!("Failed to subscribe user {} to group {}: {}", user_id, group_id, e);
                }
            } else {
                warn!("User {} tried to join group {} but connection is for group {}", 
                      user_id, requested_group_id, group_id);
                let error = WebSocketMessage::Error { 
                    message: "Can only join the group this connection is established for".to_string() 
                };
                if let Some(connection) = manager.get_connection(connection_id) {
                    let _ = connection.send_message(error).await;
                }
            }
        }
        
        WebSocketMessage::LeaveGroup { group_id: requested_group_id } => {
            if requested_group_id == group_id {
                group_service.unsubscribe_from_group(connection_id, group_id).await;
            }
        }
        
        WebSocketMessage::Ping => {
            debug!("Received ping from user {} in group {}", user_id, group_id);
            let pong = WebSocketMessage::Pong;
            let _ = manager.send_to_user(user_id, pong).await;
        }
        
        WebSocketMessage::Disconnect => {
            info!("User {} requested disconnect from group {}", user_id, group_id);
        }
        
        _ => {
            warn!("Unhandled message type from user {} in group {}: {:?}", user_id, group_id, message);
        }
    }
}
