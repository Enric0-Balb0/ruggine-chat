use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::text_message_dto::{TextMessageReadAtDtoUpdate, TextMessageSentAtDtoUpdate};

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow, Default, PartialEq, Eq)]
pub struct TextMessage {
    pub id: i32,
    pub content: String,
    pub sender_id: i32,
    pub group_chat_id: i32,
    pub sent_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct NewTextMessage {
    pub content: String,
    pub sender_id: i32,
    pub group_chat_id: i32,
}

impl NewTextMessage {
    pub fn from_dto(dto: crate::dto::text_message_dto::TextMessageCreateDto, sender_id: i32) -> Self {
        Self {
            content: dto.content,
            sender_id,
            group_chat_id: dto.group_chat_id,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct NewTextMessageInfo {
    pub user_id: i32,
    pub text_message_id: i32,
}
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct TextMessageInfo {
    pub id: i32,
    pub user_id: i32,
    pub text_message_id: i32,
    pub sent_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
}
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct TextMessageInfoUpdate {
    pub id: i32,
    pub sent_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
}


#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use super::*;
    use crate::dto::text_message_dto::TextMessageCreateDto;

    #[test]
    fn test_new_text_message_from_dto() {
        let dto = TextMessageCreateDto {
            content: "Test message content".to_string(),
            group_chat_id: 1,
        };
        let sender_id = 1;

        let new_message = NewTextMessage::from_dto(dto, sender_id);

        assert_eq!(new_message.content, "Test message content");
        assert_eq!(new_message.sender_id, sender_id);
        assert_eq!(new_message.group_chat_id, 1);
    }

    #[test]
    fn test_new_text_message_from_dto_with_different_ids() {
        let dto = TextMessageCreateDto {
            content: "Another test message".to_string(),
            group_chat_id: 10,
        };
        let sender_id = 42;

        let new_message = NewTextMessage::from_dto(dto, sender_id);

        assert_eq!(new_message.content, "Another test message");
        assert_eq!(new_message.sender_id, sender_id);
        assert_eq!(new_message.group_chat_id, 10);
    }
}