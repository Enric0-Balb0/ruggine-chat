use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::{DateTime, Utc};
use crate::entity::text_message::{TextMessage, TextMessageInfo};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate, PartialEq, Eq)]
pub struct TextMessageCreateDto {
    #[validate(length(min = 1, max = 10000, message = "Message content cannot be empty"))]
    #[schema(example = "Hello everyone! How are you doing today?")]
    pub content: String,
    #[schema(example = 1)]
    pub group_chat_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct TextMessageReadDto {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "Hello everyone! How are you doing today?")]
    pub content: String,
    #[schema(example = 1)]
    pub sender_id: i32,
    #[schema(example = 1)]
    pub group_chat_id: i32,
    #[schema(example = "2025-08-12T10:30:00Z")]
    pub sent_at: DateTime<Utc>,
}

impl From<TextMessage> for TextMessageReadDto {
    fn from(text_message: TextMessage) -> Self {
        TextMessageReadDto {
            id: text_message.id,
            content: text_message.content,
            sender_id: text_message.sender_id,
            group_chat_id: text_message.group_chat_id,
            sent_at: text_message.sent_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Validate, Deserialize, ToSchema, PartialEq, Eq)]
pub struct TextMessageInfoReadDto {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = 1)]
    pub user_id: i32,
    #[schema(example = 1)]
    pub text_message_id: i32,
    #[schema(example = "2025-08-12T10:30:00Z")]
    pub sent_at: Option<DateTime<Utc>>,
    #[schema(example = "2025-08-12T10:31:00Z")]
    pub read_at: Option<DateTime<Utc>>,
}

impl From<TextMessageInfo> for TextMessageInfoReadDto {
    fn from(info: TextMessageInfo) -> Self {
        TextMessageInfoReadDto {
            id: info.id,
            user_id: info.user_id,
            text_message_id: info.text_message_id,
            sent_at: info.sent_at,
            read_at: info.read_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Validate, Deserialize, ToSchema, PartialEq, Eq)]
pub struct TextMessageInfoCreateDto {
    #[schema(example = 1)]
    pub user_id: i32,
    #[schema(example = 1)]
    pub text_message_id: i32,
}

#[derive(Debug, Clone, Serialize, Validate, Deserialize, ToSchema, PartialEq, Eq)]
pub struct TextMessageLastReadAtDto {
    #[schema(example = 1)]
    pub group_chat_id: i32,
}

#[derive(Debug, Clone, Serialize, Validate, Deserialize, ToSchema, PartialEq, Eq)]
pub struct TextMessageLastSentAtDto {
    #[schema(example = 1)]
    pub group_chat_id: i32,
}


#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct TextMessageSentAtDtoUpdate {
    #[schema(example = 1)]
    pub text_message_id: i32,
    #[schema(example = "2025-08-12T10:30:00Z")]
    pub sent_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct TextMessageReadAtDtoUpdate {
    #[schema(example = 1)]
    pub text_message_id: i32,
    #[schema(example = "2025-08-12T10:30:00Z")]
    pub read_at: DateTime<Utc>,
}


#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;
    use chrono::Utc;

    #[test]
    fn test_text_message_create_dto_valid() {
        let create_dto = TextMessageCreateDto {
            content: "This is a valid message".to_string(),
            group_chat_id: 1,
        };

        assert!(create_dto.validate().is_ok());
        assert_eq!(create_dto.content, "This is a valid message");
        assert_eq!(create_dto.group_chat_id, 1);
    }

    #[test]
    fn test_text_message_create_dto_empty_content() {
        let create_dto = TextMessageCreateDto {
            content: "".to_string(),
            group_chat_id: 1,
        };

        let validation_result = create_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("content"));
    }

    #[test]
    fn test_text_message_read_dto_from_entity() {
        let text_message = TextMessage {
            id: 1,
            content: "Test message content".to_string(),
            sender_id: 42,
            group_chat_id: 10,
            sent_at: Utc::now(),
        };

        let read_dto = TextMessageReadDto::from(text_message.clone());

        assert_eq!(read_dto.id, text_message.id);
        assert_eq!(read_dto.content, text_message.content);
        assert_eq!(read_dto.sender_id, text_message.sender_id);
        assert_eq!(read_dto.group_chat_id, text_message.group_chat_id);
        assert_eq!(read_dto.sent_at, text_message.sent_at);
    }

    #[test]
    fn test_text_message_create_dto_long_content() {
        let long_content = "a".repeat(10000); // Very long message
        let create_dto = TextMessageCreateDto {
            content: long_content.clone(),
            group_chat_id: 5,
        };

        assert!(create_dto.validate().is_ok());
        assert_eq!(create_dto.content, long_content);
        assert_eq!(create_dto.group_chat_id, 5);
    }

    #[test]
    fn test_text_message_create_dto_not_valid_long_content() {
        let long_content = "a".repeat(10001); // Very long message
        let create_dto = TextMessageCreateDto {
            content: long_content.clone(),
            group_chat_id: 5,
        };

        assert!(create_dto.validate().is_err());
    }

    #[test]
    fn test_text_message_create_dto_with_special_characters() {
        let special_content = "Message with émojis 🎉 and special chars: àáâãäå çčć";
        let create_dto = TextMessageCreateDto {
            content: special_content.to_string(),
            group_chat_id: 3,
        };

        assert!(create_dto.validate().is_ok());
        assert_eq!(create_dto.content, special_content);
        assert_eq!(create_dto.group_chat_id, 3);
    }
}