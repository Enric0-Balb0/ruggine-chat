use serde::{Deserialize, Serialize};

/// Eventi di notifica inviati dal server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationEvent {
    Info { message: String },
    Success { message: String },
    Warning { message: String },
    Error { message: String },
}
