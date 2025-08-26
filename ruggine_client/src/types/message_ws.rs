// Stato della connessione WebSocket condiviso tra client e hooks
#[derive(Debug, Clone, PartialEq)]
pub enum WsStatus {
    Connecting,
    Open,
    Closed,
    Error(String),
}
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::Value;

// =====================
// WebSocketMessage root
// =====================
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketMessage {
    Request {
        request_id: String,
        action: ClientAction,
    },
    Response {
        request_id: String,
        ok: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<WsError>,
    },
    Event {
        event: ServerEvent,
        timestamp: DateTime<Utc>,
    },
    Control(ControlMessage),
}

// =============
// ClientAction
// =============
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientAction {
    Groups(GroupAction),
    Test { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupAction {
    Join {},
    Leave {},
    NewMessage {
        group_id: i32,
        content: String,
    },
}

// =============
// ServerEvent
// =============
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerEvent {
    Groups(GroupEvent),
    Notifications(NotificationEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupEvent {
    Joined {},
    Left {},
    NewMessage {
        message_id: i32,
        group_id: i32,
        sender_id: i32,
        sender_username: String,
        content: String,
        sent_at: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationEvent {
    Info { message: String },
    Success { message: String },
    Warning { message: String },
    Error { message: String },
}

// =============
// ControlMessage
// =============
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlMessage {
    Connect,
    Disconnect,
    Ping,
    Pong,
    Ack { message_id: Option<String> },
    Error {
        code: u16,
        message: String,
        message_id: Option<String>,
    },
    ValidationError {
        code: u16,
        message: String,
        message_id: Option<String>,
    },
}

// =============
// WsError
// =============
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsError {
    pub code: u16,
    pub message: String,
    pub message_id: Option<String>,
}
