use std::sync::{Arc, Weak};
use tokio::sync::{mpsc, watch, Mutex};
use tokio::task::JoinHandle;
use crate::websocket::message::WebSocketMessage;
use crate::error::connection_error::ConnectionError;
use tracing::{debug, error, info, warn};
use crate::websocket::WebSocketManager;

#[derive(Debug)]
pub struct WebSocketConnection {
    pub user_id: i32,
    pub connection_id: String,
    sender: mpsc::UnboundedSender<WebSocketMessage>,
    close_tx: watch::Sender<bool>,
    pub close_rx: watch::Receiver<bool>,
    handle: Mutex<Option<JoinHandle<()>>>,
    manager: Weak<WebSocketManager>,
}

impl WebSocketConnection {
    pub fn new(
        user_id: i32,
        connection_id: String,
        sender: mpsc::UnboundedSender<WebSocketMessage>,
        handle: Option<JoinHandle<()>>,
        manager: Weak<WebSocketManager>,
    ) -> Arc<Self> {
        let (close_tx, close_rx) = watch::channel(false);

        Arc::new(Self {
            user_id,
            connection_id,
            sender,
            close_tx,
            close_rx,
            handle: Mutex::new(handle),
            manager,
        })
    }

    pub async fn set_handle(&self, handle: JoinHandle<()>) {
        let mut guard = self.handle.lock().await;
        *guard = Some(handle);
    }

    pub async fn send_message(&self, message: WebSocketMessage) -> Result<(), ConnectionError> {
        self.sender.send(message).map_err(|_| ConnectionError::FailedToSendMessage)
    }

    pub async fn await_connection_tasks(
        send_task: JoinHandle<()>,
        connection: Arc<WebSocketConnection>,
        receive_task: JoinHandle<()>,
    ) {
        // Ora aggiorno il handle della connessione con il receive_task
        connection.set_handle(receive_task).await;

        // Select tra send_task e receive_task
        let mut handle_guard = connection.handle.lock().await;
        let receive_task = handle_guard.as_mut().unwrap();

        let mut close_rx = connection.close_rx.clone();

        // Se close è già stata chiamata, questo ritorna subito
        if *close_rx.borrow() {
            debug!("Connection already closed before tasks started");
            return connection.close().await;
        }

        tokio::select! {
            _ = send_task => debug!("Send task completed"),
            _ = receive_task => debug!("Receive task completed"),
            _ = close_rx.changed() => debug!("Connection closed by client"),
        }

        connection.close().await;
    }

    /// Segnala la chiusura e aspetta il task
    pub async fn close(&self) {
        let _ = self.close_tx.send(true);

        // cleanup nel manager se è ancora vivo
        if let Some(manager) = self.manager.upgrade() {
            manager
                .remove_connection(&self.connection_id, self.user_id)
                .await;
        }

        info!("Connection {} closed", self.connection_id);
    }

    /// Controlla se la connessione è ancora attiva
    pub fn is_active(&self) -> bool {
        !*self.close_tx.borrow()
    }
}
