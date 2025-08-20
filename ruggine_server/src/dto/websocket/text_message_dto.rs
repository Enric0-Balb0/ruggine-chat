use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct NewMessage { pub message_id: i32, pub group_id: i32, pub sender_id: i32, pub sender_username: String, pub content: String, pub sent_at: DateTime<Utc> }