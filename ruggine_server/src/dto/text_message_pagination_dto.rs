use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::{DateTime, Utc};

/// Pagination query specifically for text messages (uses DateTime cursor)
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct TextMessagePaginationQuery {
    /// Cursor for pagination (timestamp)
    #[schema(example = "2025-08-12T10:30:00Z")]
    pub cursor: Option<DateTime<Utc>>,
    
    /// Number of items to return (max 100)
    #[serde(default = "default_limit")]
    #[validate(range(min = 1, max = 100))]
    #[schema(example = 20, minimum = 1, maximum = 100)]
    pub limit: usize,
}

impl TextMessagePaginationQuery {
    pub fn new(cursor: Option<DateTime<Utc>>, limit: usize) -> Self {
        Self { cursor, limit }
    }
}

fn default_limit() -> usize {
    20
}

/// DTO for finding text messages by group with pagination
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct FindTextMessagesByGroupQuery {
    /// Group chat ID to filter by
    #[validate(range(min = 1))]
    pub group_chat_id: i32,
    
    /// Pagination parameters
    #[serde(flatten)]
    pub pagination: TextMessagePaginationQuery,
}

impl FindTextMessagesByGroupQuery {
    pub fn new(group_chat_id: i32, cursor: Option<DateTime<Utc>>, limit: usize) -> Self {
        Self {
            group_chat_id,
            pagination: TextMessagePaginationQuery::new(cursor, limit),
        }
    }
}
