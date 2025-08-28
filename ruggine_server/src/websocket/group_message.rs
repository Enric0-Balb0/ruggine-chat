use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Azioni client legate ai gruppi
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupAction {
    Join { },
    Leave { },
}

/// Eventi server legati ai gruppi
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupEvent {
    Joined { user_id: i32 },
    Left { user_id: i32 },
    NewMessage {
        message_id: i32,
        group_id: i32,
        sender_id: i32,
        sender_username: String,
        content: String,
        sent_at: DateTime<Utc>,
    },
}
