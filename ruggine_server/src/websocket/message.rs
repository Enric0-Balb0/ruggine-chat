use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Enum per i messaggi WebSocket scambiati tra client e server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketMessage {
    // Messaggi di controllo della connessione
    Connect,  // Rimosso user_id poiché il server lo conosce già dall'autenticazione
    Disconnect,
    Ack { message_id: Option<String> },
    Error { message: String },
    
    // Messaggi di ping/pong per keep-alive
    Ping,
    Pong,
    
    // Messaggi di test
    Test { message: String },
    
    // Messaggi di gruppo
    JoinGroup { group_id: i32 },
    LeaveGroup { group_id: i32 },
    GroupJoined { group_id: i32 },
    GroupLeft { group_id: i32 },
    
    // Nuovi messaggi di testo nel gruppo
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
pub enum GroupAction {
    Created,
    Updated,
    Deleted,
    MemberAdded,
    MemberRemoved,
    MemberRoleChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvitationAction {
    Created,
    Accepted,
    Rejected,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Tipo per messaggi dal client (subset di WebSocketMessage)
pub type ClientMessage = WebSocketMessage;

impl WebSocketMessage {
    /// Serializza il messaggio in JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    /// Deserializza un messaggio da JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
    
    /// Crea un messaggio di errore
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error { 
            message: message.into() 
        }
    }

}