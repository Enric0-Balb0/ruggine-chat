use std::sync::atomic::{AtomicU32, Ordering};
use chrono::{DateTime, Utc};
use crate::entity::text_message::{TextMessage, NewTextMessage, NewTextMessageInfo, TextMessageInfo};
use crate::dto::text_message_dto::{TextMessageCreateDto, TextMessageInfoCreateDto, TextMessageInfoReadDto, TextMessageReadDto};

// Counter per generare dati unici nei test
static TEST_COUNTER: AtomicU32 = AtomicU32::new(1);

pub struct TextMessageFactory;

impl TextMessageFactory {
    pub fn fake_content(prefix: &str) -> String {
        format!("{} message content", prefix)
    }

    pub fn get_unique_message_information(prefix: &str) -> String {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("{}_message_content_{}", prefix, counter)
    }

    // NewTextMessage factory methods
    pub fn fake_new_text_message() -> NewTextMessage {
        NewTextMessage {
            content: "Test message content".to_string(),
            sender_id: 1,
            group_chat_id: 1,
        }
    }

    pub fn fake_new_text_message_with_ids(sender_id: i32, group_chat_id: i32) -> NewTextMessage {
        NewTextMessage {
            content: "Test message content".to_string(),
            sender_id,
            group_chat_id,
        }
    }

    pub fn fake_new_text_message_with_content(content: String) -> NewTextMessage {
        NewTextMessage {
            content,
            sender_id: 1,
            group_chat_id: 1,
        }
    }

    pub fn unique_fake_new_text_message(prefix: &str, sender_id: i32, group_chat_id: i32) -> NewTextMessage {
        NewTextMessage {
            content: Self::get_unique_message_information(prefix),
            sender_id,
            group_chat_id,
        }
    }

    // TextMessage factory methods
    pub fn fake_text_message() -> TextMessage {
        TextMessage {
            id: 1,
            content: "Test message content".to_string(),
            sender_id: 1,
            group_chat_id: 1,
            sent_at: Utc::now(),
        }
    }

    pub fn fake_text_message_with_id(id: i32) -> TextMessage {
        TextMessage {
            id,
            content: "Test message content".to_string(),
            sender_id: 1,
            group_chat_id: 1,
            sent_at: Utc::now(),
        }
    }

    pub fn fake_text_message_with_ids(id: i32, sender_id: i32, group_chat_id: i32) -> TextMessage {
        TextMessage {
            id,
            content: "Test message content".to_string(),
            sender_id,
            group_chat_id,
            sent_at: Utc::now(),
        }
    }

    pub fn fake_text_message_with_content(content: String) -> TextMessage {
        TextMessage {
            id: 1,
            content,
            sender_id: 1,
            group_chat_id: 1,
            sent_at: Utc::now(),
        }
    }

    pub fn fake_text_messages_for_group(group_chat_id: i32, count: usize) -> Vec<TextMessage> {
        let mut messages = Vec::new();
        let base_time = Utc::now();
        
        for i in 0..count {
            messages.push(TextMessage {
                id: (i + 1) as i32,
                content: format!("Message {} for group {}", i + 1, group_chat_id),
                sender_id: ((i % 3) + 1) as i32, // Rotate between users 1, 2, 3
                group_chat_id,
                sent_at: base_time - chrono::Duration::minutes(i as i64),
            });
        }
        
        messages
    }

    pub fn unique_fake_text_message(prefix: &str, sender_id: i32, group_chat_id: i32) -> TextMessage {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        TextMessage {
            id: counter as i32,
            content: Self::get_unique_message_information(prefix),
            sender_id,
            group_chat_id,
            sent_at: Utc::now(),
        }
    }

    // DTO factory methods
    pub fn fake_text_message_create_dto() -> TextMessageCreateDto {
        TextMessageCreateDto {
            content: "Test message content".to_string(),
            group_chat_id: 1,
        }
    }

    pub fn fake_text_message_create_dto_with_content(content: String) -> TextMessageCreateDto {
        TextMessageCreateDto {
            content,
            group_chat_id: 1,
        }
    }

    pub fn fake_text_message_create_dto_with_group_id(group_chat_id: i32) -> TextMessageCreateDto {
        TextMessageCreateDto {
            content: "Test message content".to_string(),
            group_chat_id,
        }
    }

    pub fn unique_fake_text_message_create_dto(prefix: &str, group_chat_id: i32) -> TextMessageCreateDto {
        TextMessageCreateDto {
            content: Self::get_unique_message_information(prefix),
            group_chat_id,
        }
    }

    pub fn fake_text_message_read_dto() -> TextMessageReadDto {
        TextMessageReadDto {
            id: 1,
            content: "Test message content".to_string(),
            sender_id: 1,
            group_chat_id: 1,
            sent_at: Utc::now(),
        }
    }

    pub fn fake_text_message_read_dto_with_ids(id: i32, sender_id: i32, group_chat_id: i32) -> TextMessageReadDto {
        TextMessageReadDto {
            id,
            content: "Test message content".to_string(),
            sender_id,
            group_chat_id,
            sent_at: Utc::now(),
        }
    }

    // Utility methods for customization
    pub fn with_content(mut new_message: NewTextMessage, content: String) -> NewTextMessage {
        new_message.content = content;
        new_message
    }

    pub fn with_sender_id(mut new_message: NewTextMessage, sender_id: i32) -> NewTextMessage {
        new_message.sender_id = sender_id;
        new_message
    }

    pub fn with_group_chat_id(mut new_message: NewTextMessage, group_chat_id: i32) -> NewTextMessage {
        new_message.group_chat_id = group_chat_id;
        new_message
    }

    pub fn with_id(mut message: TextMessage, id: i32) -> TextMessage {
        message.id = id;
        message
    }

    pub fn with_sent_at(mut message: TextMessage, sent_at: DateTime<Utc>) -> TextMessage {
        message.sent_at = sent_at;
        message
    }

    pub fn with_content_dto(mut dto: TextMessageCreateDto, content: String) -> TextMessageCreateDto {
        dto.content = content;
        dto
    }

    pub fn with_group_chat_id_dto(mut dto: TextMessageCreateDto, group_chat_id: i32) -> TextMessageCreateDto {
        dto.group_chat_id = group_chat_id;
        dto
    }

    // NewTextMessageInfo factory methods
    pub fn fake_new_text_message_info() -> NewTextMessageInfo {
        NewTextMessageInfo {
            user_id: 1,
            text_message_id: 1,
        }
    }

    pub fn fake_new_text_message_info_with_ids(user_id: i32, text_message_id: i32) -> NewTextMessageInfo {
        NewTextMessageInfo {
            user_id,
            text_message_id,
        }
    }

    // Utility methods for NewTextMessageInfo
    pub fn with_user_id_info(mut info: NewTextMessageInfo, user_id: i32) -> NewTextMessageInfo {
        info.user_id = user_id;
        info
    }

    pub fn with_text_message_id_info(mut info: NewTextMessageInfo, text_message_id: i32) -> NewTextMessageInfo {
        info.text_message_id = text_message_id;
        info
    }

    // TextMessageInfo factory methods
    pub fn fake_text_message_info() -> TextMessageInfo {
        TextMessageInfo {
            id: 1,
            user_id: 1,
            text_message_id: 1,
            sent_at: Some(Utc::now()),
            read_at: None,
        }
    }

    pub fn fake_text_message_info_with_ids(id: i32, user_id: i32, text_message_id: i32) -> TextMessageInfo {
        TextMessageInfo {
            id,
            user_id,
            text_message_id,
            sent_at: Some(Utc::now()),
            read_at: None,
        }
    }

    pub fn fake_text_message_info_with_ids_and_read_at(id: i32, user_id: i32, text_message_id: i32) -> TextMessageInfo {
        TextMessageInfo {
            id,
            user_id,
            text_message_id,
            sent_at: Some(Utc::now()),
            read_at: Some(Utc::now()),
        }
    }

    pub fn fake_text_message_info_read_dto_with_ids_and_read_at(id: i32, user_id: i32, text_message_id: i32) -> TextMessageInfoReadDto {
        let now = Utc::now();
        TextMessageInfoReadDto {
            id,
            user_id,
            text_message_id,
            sent_at: Some(now - chrono::Duration::minutes(5)),
            read_at: Some(now),
        }
    }

    pub fn fake_text_message_info_sent_only(id: i32, user_id: i32, text_message_id: i32) -> TextMessageInfo {
        TextMessageInfo {
            id,
            user_id,
            text_message_id,
            sent_at: Some(Utc::now()),
            read_at: None,
        }
    }

    pub fn fake_text_message_infos_for_message(message_id: i32, count: usize) -> Vec<TextMessageInfo> {
        let mut infos = Vec::new();
        let base_time = Utc::now();
        
        for i in 0..count {
            infos.push(TextMessageInfo {
                id: (i + 1) as i32,
                user_id: ((i % 3) + 1) as i32, // Rotate between users 1, 2, 3
                text_message_id: message_id,
                sent_at: Some(base_time - chrono::Duration::minutes(i as i64)),
                read_at: if i % 2 == 0 { Some(base_time - chrono::Duration::minutes(i as i64 / 2)) } else { None },
            });
        }
        
        infos
    }

    // Utility methods for TextMessageInfo
    pub fn with_id_info(mut info: TextMessageInfo, id: i32) -> TextMessageInfo {
        info.id = id;
        info
    }

    pub fn with_user_id_full_info(mut info: TextMessageInfo, user_id: i32) -> TextMessageInfo {
        info.user_id = user_id;
        info
    }

    pub fn with_text_message_id_full_info(mut info: TextMessageInfo, text_message_id: i32) -> TextMessageInfo {
        info.text_message_id = text_message_id;
        info
    }

    pub fn with_sent_at_info(mut info: TextMessageInfo, sent_at: Option<DateTime<Utc>>) -> TextMessageInfo {
        info.sent_at = sent_at;
        info
    }

    pub fn with_read_at_info(mut info: TextMessageInfo, read_at: Option<DateTime<Utc>>) -> TextMessageInfo {
        info.read_at = read_at;
        info
    }

    pub fn fake_text_message_info_create_dto() -> TextMessageInfoCreateDto {
        TextMessageInfoCreateDto {
            user_id: 1,
            text_message_id: 1,
        }
    }

    pub fn fake_text_message_info_create_dto_with_ids(user_id: i32, text_message_id: i32) -> TextMessageInfoCreateDto {
        TextMessageInfoCreateDto {
            user_id,
            text_message_id,
        }
    }

    // Update DTO factory methods
    pub fn fake_text_message_read_at_dto_update() -> crate::dto::text_message_dto::TextMessageReadAtDtoUpdate {
        use crate::dto::text_message_dto::TextMessageReadAtDtoUpdate;
        TextMessageReadAtDtoUpdate {
            text_message_id: 1,
            read_at: Utc::now(),
        }
    }

    pub fn fake_text_message_read_at_dto_update_with_ids(text_message_id: i32, read_at: DateTime<Utc>) -> crate::dto::text_message_dto::TextMessageReadAtDtoUpdate {
        use crate::dto::text_message_dto::TextMessageReadAtDtoUpdate;
        TextMessageReadAtDtoUpdate {
            text_message_id,
            read_at,
        }
    }

    pub fn fake_text_message_sent_at_dto_update() -> crate::dto::text_message_dto::TextMessageSentAtDtoUpdate {
        use crate::dto::text_message_dto::TextMessageSentAtDtoUpdate;
        TextMessageSentAtDtoUpdate {
            text_message_id: 1,
            sent_at: Utc::now(),
        }
    }

    pub fn fake_text_message_sent_at_dto_update_with_ids(text_message_id: i32, sent_at: DateTime<Utc>) -> crate::dto::text_message_dto::TextMessageSentAtDtoUpdate {
        use crate::dto::text_message_dto::TextMessageSentAtDtoUpdate;
        TextMessageSentAtDtoUpdate {
            text_message_id,
            sent_at,
        }
    }
}
