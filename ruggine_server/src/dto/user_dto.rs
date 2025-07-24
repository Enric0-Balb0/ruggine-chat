use crate::entity::user::User;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Validate)]
pub struct UserLoginDto {
    #[validate(email(message = "Email is not valid"))]
    pub email: String,
    #[validate(length(
        min = 3,
        max = 50,
        message = "Password must be between 3 and 50 characters"
    ))]
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize, Validate, PartialEq, Eq)]
pub struct UserRegisterDto {
    #[validate(email(message = "Email is not valid"))]
    pub email: String,
    #[validate(length(
        min = 3,
        max = 50,
        message = "Password must be between 3 and 50 characters"
    ))]
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    pub user_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UserReadDto {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub user_name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: i8,
}

impl UserReadDto {
    pub fn from(model: User) -> UserReadDto {
        Self {
            id: model.id,
            first_name: model.first_name,
            last_name: model.last_name,
            user_name: model.user_name,
            email: model.email,
            created_at: model.created_at,
            updated_at: model.updated_at,
            is_active: model.is_active,
        }
    }
}

impl std::fmt::Debug for UserLoginDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("email", &self.email)
            .finish()
    }
}

impl std::fmt::Debug for UserRegisterDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("first_name", &self.first_name)
            .field("last_name", &self.last_name)
            .field("user_name", &self.user_name)
            .field("email", &self.email)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_user_login_dto_valid() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        assert!(login_dto.validate().is_ok());
        assert_eq!(login_dto.email, "test@example.com");
        assert_eq!(login_dto.password, "password123");
    }

    #[test]
    fn test_user_login_dto_invalid_email() {
        let login_dto = UserLoginDto {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
        };

        let validation_result = login_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("email"));
    }

    #[test]
    fn test_user_login_dto_password_too_short() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "12".to_string(), // Less than 3 characters
        };

        let validation_result = login_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("password"));
    }

    #[test]
    fn test_user_login_dto_password_too_long() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "a".repeat(51), // More than 50 characters
        };

        let validation_result = login_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("password"));
    }

    #[test]
    fn test_user_register_dto_valid() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
        };

        assert!(register_dto.validate().is_ok());
        assert_eq!(register_dto.email, "test@example.com");
        assert_eq!(register_dto.password, "password123");
        assert_eq!(register_dto.first_name, "John");
        assert_eq!(register_dto.last_name, "Doe");
        assert_eq!(register_dto.user_name, "johndoe");
    }

    #[test]
    fn test_user_register_dto_invalid_email() {
        let register_dto = UserRegisterDto {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("email"));
    }

    #[test]
    fn test_user_register_dto_password_too_short() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "12".to_string(), // Less than 3 characters
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("password"));
    }

    #[test]
    fn test_user_register_dto_password_too_long() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "a".repeat(51), // More than 50 characters
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("password"));
    }

    #[test]
    fn test_user_register_dto_username_too_short() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "ab".to_string(), // Less than 3 characters
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("user_name"));
    }

    #[test]
    fn test_user_register_dto_username_too_long() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "a".repeat(51), // More than 50 characters
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("user_name"));
    }

    #[test]
    fn test_user_login_dto_debug_format() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let debug_string = format!("{:?}", login_dto);
        assert!(debug_string.contains("test@example.com"));
        // Password should not be shown in debug format for security
        assert!(!debug_string.contains("password123"));
    }

    #[test]
    fn test_user_register_dto_debug_format() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
        };

        let debug_string = format!("{:?}", register_dto);
        assert!(debug_string.contains("John"));
        assert!(debug_string.contains("Doe"));
        assert!(debug_string.contains("johndoe"));
        assert!(debug_string.contains("test@example.com"));
        // Password should not be shown in debug format for security
        assert!(!debug_string.contains("password123"));
    }

    #[test]
    fn test_user_login_dto_clone() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let cloned_dto = login_dto.clone();
        assert_eq!(login_dto.email, cloned_dto.email);
        assert_eq!(login_dto.password, cloned_dto.password);
    }

    #[test]
    fn test_user_register_dto_clone() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
        };

        let cloned_dto = register_dto.clone();
        assert_eq!(register_dto.email, cloned_dto.email);
        assert_eq!(register_dto.password, cloned_dto.password);
        assert_eq!(register_dto.first_name, cloned_dto.first_name);
        assert_eq!(register_dto.last_name, cloned_dto.last_name);
        assert_eq!(register_dto.user_name, cloned_dto.user_name);
    }

    #[test]
    fn test_user_login_dto_serialization() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        // Test serialization
        let serialized = serde_json::to_string(&login_dto).unwrap();
        assert!(serialized.contains("test@example.com"));
        assert!(serialized.contains("password123"));

        // Test deserialization
        let deserialized: UserLoginDto = serde_json::from_str(&serialized).unwrap();
        assert_eq!(login_dto.email, deserialized.email);
        assert_eq!(login_dto.password, deserialized.password);
    }

    #[test]
    fn test_user_register_dto_serialization() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
        };

        // Test serialization
        let serialized = serde_json::to_string(&register_dto).unwrap();
        assert!(serialized.contains("test@example.com"));
        assert!(serialized.contains("John"));

        // Test deserialization
        let deserialized: UserRegisterDto = serde_json::from_str(&serialized).unwrap();
        assert_eq!(register_dto.email, deserialized.email);
        assert_eq!(register_dto.first_name, deserialized.first_name);
        assert_eq!(register_dto.last_name, deserialized.last_name);
        assert_eq!(register_dto.user_name, deserialized.user_name);
    }
}