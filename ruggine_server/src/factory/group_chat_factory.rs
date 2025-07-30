use std::sync::{atomic::{AtomicU32, Ordering}};
use chrono::Utc;

use crate::{
    dto::group_chat_dto::{GroupChatCreateDto, GroupChatReadDto, GroupChatUpdateDto},
    entity::group_chat::{NewGroupChat, GroupChat, UpdateGroupChat}
};

// Global counter for unique test data
static TEST_COUNTER: AtomicU32 = AtomicU32::new(1);

pub struct GroupChatFactory;

impl GroupChatFactory {
    pub fn fake_group_name(prefix: &str) -> String {
        format!("{}_group", prefix)
    }

    pub fn fake_description(prefix: &str) -> String {
        format!("This is a test group for {}", prefix)
    }

    pub fn get_unique_group_information(prefix: &str) -> (String, String) {
        let counter: u32 = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        let unique_suffix = format!("{}_{}", counter, timestamp);
        let name = format!("{}_{}", prefix, unique_suffix);
        let description = format!("Test group for {} with ID {}", prefix, unique_suffix);
        
        (name, description)
    }

    pub fn unique_fake_new_group_chat(prefix: &str, created_by: i32) -> NewGroupChat {
        let (name, description) = Self::get_unique_group_information(prefix);
        NewGroupChat {
            name,
            description,
            created_by,
        }
    }

    pub fn fake_new_group_chat() -> NewGroupChat {
        NewGroupChat {
            name: "Test Group".to_string(),
            description: "This is a test group description".to_string(),
            created_by: 1,
        }
    }

    pub fn fake_group_chat() -> GroupChat {
        GroupChat {
            id: 1,
            name: "Test Group".to_string(),
            description: "This is a test group description".to_string(),
            created_by: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn fake_group_chat_create_dto() -> GroupChatCreateDto {
        GroupChatCreateDto {
            name: "Test Group".to_string(),
            description: "This is a test group description".to_string(),
        }
    }

    pub fn unique_fake_group_chat_create_dto(prefix: &str) -> GroupChatCreateDto {
        let (name, description) = Self::get_unique_group_information(prefix);
        GroupChatCreateDto {
            name,
            description,
        }
    }

    pub fn fake_group_chat_read_dto() -> GroupChatReadDto {
        use chrono::{NaiveDate, DateTime, Utc};

        let naive_dt = NaiveDate::from_ymd_opt(2025, 7, 29)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();

        GroupChatReadDto {
            id: 1,
            name: "Test Group".to_string(),
            description: "This is a test group description".to_string(),
            created_by: 1,
            created_at: DateTime::from_naive_utc_and_offset(naive_dt, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(naive_dt, Utc),
        }
    }

    pub fn fake_group_chat_update_dto() -> GroupChatUpdateDto {
        GroupChatUpdateDto {
            name: Some("Updated Test Group".to_string()),
            description: Some("Updated test group description".to_string()),
        }
    }

    pub fn unique_fake_group_chat_update_dto(prefix: &str) -> GroupChatUpdateDto {
        let (name, description) = Self::get_unique_group_information(&format!("updated_{}", prefix));
        GroupChatUpdateDto {
            name: Some(name),
            description: Some(description),
        }
    }

    pub fn fake_update_group_chat() -> UpdateGroupChat {
        UpdateGroupChat {
            name: Some("Updated Test Group".to_string()),
            description: Some("Updated test group description".to_string()),
            updated_at: Utc::now(),
        }
    }

    pub fn unique_fake_update_group_chat(prefix: &str) -> UpdateGroupChat {
        let (name, description) = Self::get_unique_group_information(&format!("updated_{}", prefix));
        UpdateGroupChat {
            name: Some(name),
            description: Some(description),
            updated_at: Utc::now(),
        }
    }

    // Utility methods for testing
    pub fn with_specific_creator(mut new_group: NewGroupChat, created_by: i32) -> NewGroupChat {
        new_group.created_by = created_by;
        new_group
    }

    pub fn with_name(mut new_group: NewGroupChat, name: String) -> NewGroupChat {
        new_group.name = name;
        new_group
    }

    pub fn with_description(mut new_group: NewGroupChat, description: String) -> NewGroupChat {
        new_group.description = description;
        new_group
    }

    // Utility methods for DTOs
    pub fn with_name_dto(mut dto: GroupChatCreateDto, name: String) -> GroupChatCreateDto {
        dto.name = name;
        dto
    }

    pub fn with_description_dto(mut dto: GroupChatCreateDto, description: String) -> GroupChatCreateDto {
        dto.description = description;
        dto
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_group_name() {
        let name = GroupChatFactory::fake_group_name("test");
        assert_eq!(name, "test_group");
    }

    #[test]
    fn test_fake_description() {
        let desc = GroupChatFactory::fake_description("test");
        assert_eq!(desc, "This is a test group for test");
    }

    #[test]
    fn test_unique_fake_new_group_chat() {
        let group1 = GroupChatFactory::unique_fake_new_group_chat("test", 1);
        let group2 = GroupChatFactory::unique_fake_new_group_chat("test", 1);

        assert_ne!(group1.name, group2.name);
        assert_ne!(group1.description, group2.description);
        assert_eq!(group1.created_by, group2.created_by);
    }

    #[test]
    fn test_fake_new_group_chat() {
        let group = GroupChatFactory::fake_new_group_chat();
        assert_eq!(group.name, "Test Group");
        assert_eq!(group.description, "This is a test group description");
        assert_eq!(group.created_by, 1);
    }

    #[test]
    fn test_fake_group_chat() {
        let group = GroupChatFactory::fake_group_chat();
        assert_eq!(group.id, 1);
        assert_eq!(group.name, "Test Group");
        assert_eq!(group.created_by, 1);
    }

    #[test]
    fn test_utility_methods() {
        let mut group = GroupChatFactory::fake_new_group_chat();

        group = GroupChatFactory::with_specific_creator(group, 42);
        assert_eq!(group.created_by, 42);

        group = GroupChatFactory::with_name(group, "Custom Name".to_string());
        assert_eq!(group.name, "Custom Name");

        group = GroupChatFactory::with_description(group, "Custom Description".to_string());
        assert_eq!(group.description, "Custom Description");
    }
}
