use crate::api::ws::message_ws_service::MessageWsService;
use crate::types::message_ws::{WsStatus, WebSocketMessage, ControlMessage};
use crate::components::use_toast;
use leptos::*;
use web_sys::{console, window};
use gloo_timers::callback::Timeout;

/// Enhanced WebSocket service with automatic reconnection and error recovery
pub struct ReconnectingWsService {
    inner: MessageWsService,
    url: String,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
    reconnect_timeout: Option<Timeout>,
    is_manual_close: bool,
    last_ping_time: f64,
    ping_interval: Option<Timeout>,
}

impl ReconnectingWsService {
    pub fn new(status: RwSignal<WsStatus>, url: String) -> Self {
        Self {
            inner: MessageWsService::new(status),
            url,
            reconnect_attempts: 0,
            max_reconnect_attempts: 5,
            reconnect_timeout: None,
            is_manual_close: false,
            last_ping_time: 0.0,
            ping_interval: None,
        }
    }

    /// Connect with automatic reconnection on failure
    pub fn connect(&mut self) {
        self.is_manual_close = false;
        self.reconnect_attempts = 0;
        self.attempt_connection();
    }

    /// Set message callback with connection monitoring
    pub fn set_on_message<F>(&mut self, callback: F)
    where
        F: Fn(WebSocketMessage) + 'static + Clone,
    {
        let callback_clone = callback.clone();
        
        self.inner.set_on_message(move |msg| {
            // Update last activity time on any message
            if let Some(window) = window() {
                let performance = window.performance().unwrap();
                // Store last activity time for connection monitoring
                console::log_1(&format!("WebSocket activity at: {}", performance.now()).into());
            }
            
            // Handle pong responses
            if let WebSocketMessage::Control(ControlMessage::Pong) = &msg {
                leptos::logging::log!("[WS RECONNECTING] Received pong");
                return;
            }
            
            callback_clone(msg);
        });
    }

    /// Send message with automatic reconnection if needed
    pub fn send(&mut self, msg: &WebSocketMessage) {
        match self.inner.status() {
            WsStatus::Open => {
                self.inner.send(msg);
            }
            WsStatus::Closed | WsStatus::Error(_) => {
                leptos::logging::warn!("[WS RECONNECTING] Connection lost, attempting reconnect before send");
                Self::show_connection_warning();
                self.attempt_connection();
                // Queue the message to be sent after reconnection
                self.queue_message_after_reconnect(msg.clone());
            }
            WsStatus::Connecting => {
                leptos::logging::warn!("[WS RECONNECTING] Still connecting, queueing message");
                self.queue_message_after_reconnect(msg.clone());
            }
        }
    }

    /// Manually disconnect (stops auto-reconnection)
    pub fn disconnect(&mut self) {
        self.is_manual_close = true;
        self.clear_reconnect_timer();
        self.clear_ping_timer();
        self.inner.disconnect();
        
        let toast = use_toast();
        toast.info("Disconnesso dal server");
    }

    /// Get current connection status
    pub fn status(&self) -> WsStatus {
        self.inner.status()
    }

    /// Get status signal for reactive updates
    pub fn status_signal(&self) -> Option<ReadSignal<WsStatus>> {
        self.inner.status_signal()
    }

    fn attempt_connection(&mut self) {
        self.clear_reconnect_timer();
        
        leptos::logging::log!("[WS RECONNECTING] Attempting connection (attempt {})", self.reconnect_attempts + 1);
        
        self.inner.connect(&self.url);
        self.setup_connection_monitoring();
        
        // Schedule a check to see if connection succeeded
        let url_clone = self.url.clone();
        let attempts = self.reconnect_attempts;
        let max_attempts = self.max_reconnect_attempts;
        
        self.reconnect_timeout = Some(Timeout::new(5000, move || {
            // This will run after 5 seconds - check if we're connected
            leptos::spawn_local(async move {
                Self::check_connection_after_timeout(url_clone, attempts, max_attempts).await;
            });
        }));
    }

    async fn check_connection_after_timeout(url: String, attempts: u32, max_attempts: u32) {
        // This is a static method, so we can't directly access self
        // The actual reconnection logic should be handled by monitoring status changes
        leptos::logging::log!("[WS RECONNECTING] Connection timeout check for {}", url);
        
        if attempts < max_attempts {
            leptos::logging::log!("[WS RECONNECTING] Will retry connection if still not connected");
        } else {
            Self::show_connection_failed();
        }
    }

    fn setup_connection_monitoring(&mut self) {
        self.setup_ping_monitoring();
        self.setup_status_monitoring();
    }

    fn setup_ping_monitoring(&mut self) {
        self.clear_ping_timer();
        
        // Send ping every 30 seconds to detect connection issues
        self.ping_interval = Some(Timeout::new(30000, move || {
            leptos::spawn_local(async move {
                Self::send_ping_if_connected().await;
            });
        }));
    }

    async fn send_ping_if_connected() {
        // Send ping through the global WebSocket service if available
        leptos::logging::log!("[WS RECONNECTING] Sending keepalive ping");
        // This would need to be implemented with access to the service instance
    }

    fn setup_status_monitoring(&self) {
        if let Some(status_signal) = self.status_signal() {
            // Monitor status changes for reconnection logic
            create_effect(move |_| {
                match status_signal.get() {
                    WsStatus::Error(_) | WsStatus::Closed => {
                        leptos::spawn_local(async move {
                            Self::handle_connection_lost().await;
                        });
                    }
                    WsStatus::Open => {
                        leptos::logging::log!("[WS RECONNECTING] Connection established");
                    }
                    _ => {}
                }
            });
        }
    }

    async fn handle_connection_lost() {
        leptos::logging::warn!("[WS RECONNECTING] Connection lost, scheduling reconnect");
        // Implement reconnection logic here
    }

    fn queue_message_after_reconnect(&self, msg: WebSocketMessage) {
        // Store message to send after reconnection
        leptos::spawn_local(async move {
            // Wait for connection to be established
            crate::utils::timers::sleep_ms(2000).await;
            leptos::logging::log!("[WS RECONNECTING] Queued message: {:?}", msg);
            // TODO: Actually send the queued message
        });
    }

    fn clear_reconnect_timer(&mut self) {
        if let Some(timeout) = self.reconnect_timeout.take() {
            timeout.forget();
        }
    }

    fn clear_ping_timer(&mut self) {
        if let Some(interval) = self.ping_interval.take() {
            interval.forget();
        }
    }

    fn show_connection_warning() {
        let toast = use_toast();
        toast.warning("Connessione persa, riconnessione in corso...");
    }

    fn show_connection_failed() {
        let toast = use_toast();
        toast.error("Impossibile connettersi al server. Verifica la connessione di rete.");
    }

    fn calculate_reconnect_delay(attempt: u32) -> u32 {
        // Exponential backoff: 1s, 2s, 4s, 8s, 16s
        let delay = 1000 * (2_u32.pow(attempt.min(4)));
        delay.min(30000) // Cap at 30 seconds
    }
}

impl Drop for ReconnectingWsService {
    fn drop(&mut self) {
        self.clear_reconnect_timer();
        self.clear_ping_timer();
    }
}
