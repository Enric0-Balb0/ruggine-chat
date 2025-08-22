use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::Value;
use crate::websocket::{GroupAction, GroupEvent, NotificationEvent};

/// Messaggi WebSocket scambiati tra client e server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketMessage {
    /// Richieste client → server
    Request {
        request_id: String,
        action: ClientAction,
    },

    /// Risposte server → client
    Response {
        request_id: String,
        ok: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<WsError>,
    },

    /// Eventi push inviati dal server
    Event {
        event: ServerEvent,
        timestamp: DateTime<Utc>,
    },

    /// Messaggi di controllo / keep-alive
    Control(ControlMessage),
}

/// Parametri della query per la connessione WebSocket
#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    pub token: Option<String>,
}

/// Azioni inviate dal client, divise per dominio
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientAction {
    Groups(GroupAction),
    // in futuro: Machines(MachineAction), ecc.
    Test { message: String },
}

/// Eventi inviati dal server, divisi per dominio
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerEvent {
    Groups(GroupEvent),
    Notifications(NotificationEvent),
    // in futuro: Machines(MachineEvent), ecc.
}

/// Messaggi di controllo connessione
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

/// Errori standardizzati inviati al client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsError {
    pub code: u16,
    pub message: String,
}

impl WebSocketMessage {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn error(request_id: String, code: u16, message: impl Into<String>) -> Self {
        WebSocketMessage::Response {
            request_id,
            ok: false,
            data: None,
            error: Some(WsError {
                code,
                message: message.into(),
            }),
        }
    }

    pub fn success(request_id: String, data: impl Serialize) -> Self {
        WebSocketMessage::Response {
            request_id,
            ok: true,
            data: Some(serde_json::to_value(data).unwrap()),
            error: None,
        }
    }
}
