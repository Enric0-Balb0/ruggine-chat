use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate, PartialEq, Eq)]
pub struct GroupChatCreateDto {
    #[validate(length(min = 1, max = 256, message = "Group name must be between 1 and 256 characters"))]
    #[schema(example = "New Group")]
    pub name: String,
    
    #[validate(length(min = 1, max = 1024, message = "Group description must be between 1 and 1024 characters"))]
    #[schema(example = "New Group is the best group in the world.")]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct GroupChatReadDto {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "Test Group")]
    pub name: String,
    #[schema(example = "This is a test group description")]
    pub description: String,
    #[schema(example = 42)]
    pub created_by: i32,
    #[schema(example = "2025-07-29T10:00:00Z")]
    pub created_at: NaiveDateTime,
    #[schema(example = "2025-07-29T10:00:00Z")]
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate, PartialEq, Eq)]
pub struct GroupChatUpdateDto {
    #[validate(length(min = 1, max = 256, message = "Group name must be between 1 and 256 characters"))]
    #[schema(example = "New Group Update")]
    pub name: Option<String>,
    
    #[validate(length(min = 1, max = 1024, message = "Group description must be between 1 and 1024 characters"))]
    #[schema(example = "New Group was the second best group in the world, now it is the best one.")]
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_group_chat_create_dto_valid() {
        let create_dto = GroupChatCreateDto {
            name: "Test Group".to_string(),
            description: "This is a test group description".to_string(),
        };

        assert!(create_dto.validate().is_ok());
        assert_eq!(create_dto.name, "Test Group");
        assert_eq!(create_dto.description, "This is a test group description");
    }

    #[test]
    fn test_group_chat_create_dto_name_empty() {
        let create_dto = GroupChatCreateDto {
            name: "".to_string(),
            description: "Valid description".to_string(),
        };

        let validation_result = create_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("name"));
    }

    #[test]
    fn test_group_chat_create_dto_name_too_long() {
        let long_name = "a".repeat(257); // 257 characters, exceeds limit of 256
        let create_dto = GroupChatCreateDto {
            name: long_name,
            description: "Valid description".to_string(),
        };

        let validation_result = create_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("name"));
    }

    #[test]
    fn test_group_chat_create_dto_name_max_length() {
        let max_name = "a".repeat(256); // Exactly 256 characters
        let create_dto = GroupChatCreateDto {
            name: max_name.clone(),
            description: "Valid description".to_string(),
        };

        assert!(create_dto.validate().is_ok());
        assert_eq!(create_dto.name, max_name);
    }

    #[test]
    fn test_group_chat_create_dto_description_empty() {
        let create_dto = GroupChatCreateDto {
            name: "Valid Name".to_string(),
            description: "".to_string(),
        };

        let validation_result = create_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("description"));
    }

    #[test]
    fn test_group_chat_create_dto_description_too_long() {
        let long_description = "a".repeat(1025); // 1025 characters, exceeds limit of 1024
        let create_dto = GroupChatCreateDto {
            name: "Valid Name".to_string(),
            description: long_description,
        };

        let validation_result = create_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("description"));
    }

    #[test]
    fn test_group_chat_create_dto_description_max_length() {
        let max_description = "a".repeat(1024); // Exactly 1024 characters
        let create_dto = GroupChatCreateDto {
            name: "Valid Name".to_string(),
            description: max_description.clone(),
        };

        assert!(create_dto.validate().is_ok());
        assert_eq!(create_dto.description, max_description);
    }

    #[test]
    fn test_group_chat_create_dto_both_fields_invalid() {
        let create_dto = GroupChatCreateDto {
            name: "".to_string(), // Empty name
            description: "a".repeat(1025), // Too long description
        };

        let validation_result = create_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("name"));
        assert!(errors.field_errors().contains_key("description"));
    }

    #[test]
    fn test_group_chat_create_dto_clone() {
        let create_dto = GroupChatCreateDto {
            name: "Test Group".to_string(),
            description: "Test description".to_string(),
        };

        let cloned_dto = create_dto.clone();
        assert_eq!(create_dto.name, cloned_dto.name);
        assert_eq!(create_dto.description, cloned_dto.description);
    }

    #[test]
    fn test_group_chat_create_dto_debug_format() {
        let create_dto = GroupChatCreateDto {
            name: "Test Group".to_string(),
            description: "Test description".to_string(),
        };

        let debug_string = format!("{:?}", create_dto);
        assert!(debug_string.contains("Test Group"));
        assert!(debug_string.contains("Test description"));
        assert!(debug_string.contains("GroupChatCreateDto"));
    }

    #[test]
    fn test_group_chat_create_dto_serialization() {
        let create_dto = GroupChatCreateDto {
            name: "Test Group".to_string(),
            description: "Test description".to_string(),
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&create_dto).unwrap();
        assert!(json.contains("Test Group"));
        assert!(json.contains("Test description"));

        // Test deserialization from JSON
        let deserialized: GroupChatCreateDto = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, create_dto.name);
        assert_eq!(deserialized.description, create_dto.description);
    }

    #[test]
    fn test_group_chat_update_dto_valid() {
        let update_dto = GroupChatUpdateDto {
            name: Some("Updated Group".to_string()),
            description: Some("Updated description".to_string()),
        };

        assert!(update_dto.validate().is_ok());
        assert_eq!(update_dto.name, Some("Updated Group".to_string()));
        assert_eq!(update_dto.description, Some("Updated description".to_string()));
    }

    #[test]
    fn test_group_chat_update_dto_partial_update() {
        let update_dto = GroupChatUpdateDto {
            name: Some("Updated Group".to_string()),
            description: None,
        };

        assert!(update_dto.validate().is_ok());
        assert_eq!(update_dto.name, Some("Updated Group".to_string()));
        assert_eq!(update_dto.description, None);
    }

    #[test]
    fn test_group_chat_update_dto_empty_name() {
        let update_dto = GroupChatUpdateDto {
            name: Some("".to_string()),
            description: Some("Valid description".to_string()),
        };

        let validation_result = update_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("name"));
    }

    #[test]
    fn test_group_chat_update_dto_name_too_long() {
        let long_name = "a".repeat(257);
        let update_dto = GroupChatUpdateDto {
            name: Some(long_name),
            description: Some("Valid description".to_string()),
        };

        let validation_result = update_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("name"));
    }

    #[test]
    fn test_group_chat_update_dto_description_too_long() {
        let long_description = "a".repeat(1025);
        let update_dto = GroupChatUpdateDto {
            name: Some("Valid Name".to_string()),
            description: Some(long_description),
        };

        let validation_result = update_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("description"));
    }

    #[test]
    fn test_group_chat_read_dto_creation() {
        use chrono::NaiveDate;
        
        let read_dto = GroupChatReadDto {
            id: 1,
            name: "Test Group".to_string(),
            description: "Test description".to_string(),
            created_by: 42,
            created_at: NaiveDate::from_ymd_opt(2025, 7, 29).unwrap().and_hms_opt(10, 0, 0).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2025, 7, 29).unwrap().and_hms_opt(10, 0, 0).unwrap(),
        };

        assert_eq!(read_dto.id, 1);
        assert_eq!(read_dto.name, "Test Group");
        assert_eq!(read_dto.description, "Test description");
        assert_eq!(read_dto.created_by, 42);
        assert_eq!(read_dto.created_at.date(), NaiveDate::from_ymd_opt(2025, 7, 29).unwrap());
        assert_eq!(read_dto.updated_at.date(), NaiveDate::from_ymd_opt(2025, 7, 29).unwrap());
    }

    #[test]
    fn test_group_chat_read_dto_serialization() {
        use chrono::NaiveDate;
        
        let read_dto = GroupChatReadDto {
            id: 1,
            name: "Test Group".to_string(),
            description: "Test description".to_string(),
            created_by: 42,
            created_at: NaiveDate::from_ymd_opt(2025, 7, 29).unwrap().and_hms_opt(10, 0, 0).unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2025, 7, 29).unwrap().and_hms_opt(10, 0, 0).unwrap(),
        };

        // Test serialization to JSON
        let json = serde_json::to_string(&read_dto).unwrap();
        assert!(json.contains("Test Group"));
        assert!(json.contains("Test description"));
        assert!(json.contains("42"));
        assert!(json.contains("2025-07-29"));

        // Test deserialization from JSON
        let deserialized: GroupChatReadDto = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, read_dto.id);
        assert_eq!(deserialized.name, read_dto.name);
        assert_eq!(deserialized.description, read_dto.description);
        assert_eq!(deserialized.created_by, read_dto.created_by);
        assert_eq!(deserialized.created_at, read_dto.created_at);
        assert_eq!(deserialized.updated_at, read_dto.updated_at);
    }
}
