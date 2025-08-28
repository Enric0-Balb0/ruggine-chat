// message_ws.rs is a sibling, not a submodule. Import it at the crate::types level.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// =============================================================================
// DTOs - Server synchronized
// =============================================================================

/// Text message creation request - exact server DTO (TextMessageCreateDto)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMessageCreateRequest {
    pub content: String,
    pub group_chat_id: i32,
}

/// DTO per update read_at (TextMessageInfoReadAtDtoUpdate)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMessageInfoReadAtDtoUpdate {
    pub text_message_id: i32,
    pub read_at: String, // date-time
}

/// DTO risposta update lettura (ApiSuccessResponseUserReadDto, solo se serve lato client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSuccessResponseUserReadDto {
    pub data: UserReadDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReadDto {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub created_at: String,
    pub updated_at: String,
    pub user_status: String,
    pub user_type: String,
    pub birthday: String,
    pub is_online: bool,
    pub address: String,
    pub gender: String,
}

/// Text message read DTO from server (TextMessageReadDto)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMessageReadDto {
    pub id: i32,
    pub content: String,
    pub sender_id: i32,
    pub group_chat_id: i32,
    pub sent_at: String, // date-time format from OpenAPI
}

/// Paginated response from server (PaginatedTextMessageResponse)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedTextMessageResponse {
    pub data: Vec<TextMessageReadDto>,
    pub pagination: PaginationMetadataDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMetadataDto {
    pub has_more: bool,
    pub next_cursor: Option<String>, // date-time format
    pub page_size: i32,
    pub total_count: Option<i32>,
}

// =============================================================================
// CLIENT TYPES - Rich types with business logic
// =============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: i32,
    pub content: String,
    pub sender_id: i32,
    pub group_chat_id: i32,
    pub sent_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessagePage {
    pub data: Vec<Message>,
    pub pagination: PaginationMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaginationMetadata {
    pub has_more: bool,
    pub next_cursor: Option<DateTime<Utc>>,
    pub page_size: i32,
    pub total_count: Option<i32>,
}

// =============================================================================
// CONVERSIONS - From server DTOs
// =============================================================================

impl From<TextMessageReadDto> for Message {
    fn from(dto: TextMessageReadDto) -> Self {
        Self {
            id: dto.id,
            content: dto.content,
            sender_id: dto.sender_id,
            group_chat_id: dto.group_chat_id,
            sent_at: dto.sent_at.parse().unwrap_or_else(|_| Utc::now()),
        }
    }
}

impl From<PaginatedTextMessageResponse> for MessagePage {
    fn from(resp: PaginatedTextMessageResponse) -> Self {
        Self {
            data: resp.data.into_iter().map(Message::from).collect(),
            pagination: PaginationMetadata {
                has_more: resp.pagination.has_more,
                next_cursor: resp.pagination.next_cursor.and_then(|s| s.parse().ok()),
                page_size: resp.pagination.page_size,
                total_count: resp.pagination.total_count,
            },
        }
    }
}
