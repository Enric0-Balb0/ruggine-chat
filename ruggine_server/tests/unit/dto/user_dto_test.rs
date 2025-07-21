use chrono::{Utc};
use ruggine_server::dto::user_dto::{UserLoginDto, UserRegisterDto, UserReadDto};
use ruggine_server::entity::user::User;
use serde_json;
use validator::Validate;

#[cfg(test)]
mod user_login_dto_unit_tests {
    use super::*;

    // UserLoginDto Tests
    #[test]
    fn test_user_login_dto_valid_creation() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        assert_eq!(login_dto.email, "test@example.com");
        assert_eq!(login_dto.password, "password123");
    }

    #[test]
    fn test_user_login_dto_validation_valid_email() {
        let login_dto = UserLoginDto {
            email: "valid.email@example.com".to_string(),
            password: "validpass".to_string(),
        };

        assert!(login_dto.validate().is_ok());
    }

    #[test]
    fn test_user_login_dto_validation_invalid_email() {
        let login_dto = UserLoginDto {
            email: "invalid-email".to_string(),
            password: "validpass".to_string(),
        };

        let validation_result = login_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("email"));
    }

    #[test]
    fn test_user_login_dto_validation_password_too_short() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "ab".to_string(), // Too short
        };

        let validation_result = login_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("password"));
    }

    #[test]
    fn test_user_login_dto_validation_password_too_long() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "a".repeat(21), // Too long
        };

        let validation_result = login_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("password"));
    }

    #[test]
    fn test_user_login_dto_debug_format() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "secret123".to_string(),
        };

        let debug_str = format!("{:?}", login_dto);
        assert!(debug_str.contains("test@example.com"));
        assert!(!debug_str.contains("secret123")); // Password should not be in debug output
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
    fn test_user_login_dto_serialization() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let serialized = serde_json::to_string(&login_dto).unwrap();
        let deserialized: UserLoginDto = serde_json::from_str(&serialized).unwrap();

        assert_eq!(login_dto.email, deserialized.email);
        assert_eq!(login_dto.password, deserialized.password);
    }

        // Edge case and boundary tests
    #[test]
    fn test_user_login_dto_edge_case_emails() {
        let test_cases = vec![
            "user@domain.com",
            "user.name@domain.com",
            "user+tag@domain.com",
            "user123@domain123.com",
        ];

        for email in test_cases {
            let login_dto = UserLoginDto {
                email: email.to_string(),
                password: "validpass".to_string(),
            };
            assert!(login_dto.validate().is_ok(), "Failed for email: {}", email);
        }
    }

    #[test]
    fn test_user_register_dto_boundary_lengths() {
        // Test minimum valid lengths
        let register_dto = UserRegisterDto {
            email: "a@b.c".to_string(),
            password: "abc".to_string(), // Minimum 3 chars
            first_name: "A".to_string(),
            last_name: "B".to_string(),
            user_name: "abc".to_string(), // Minimum 3 chars
        };
        assert!(register_dto.validate().is_ok());

        // Test maximum valid lengths
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "a".repeat(20), // Maximum 20 chars
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "a".repeat(20), // Maximum 20 chars
        };
        assert!(register_dto.validate().is_ok());
    }

    #[test]
    fn test_user_read_dto_active_inactive_states() {
        let now = Utc::now();
        
        // Test active user
        let active_user = User {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
            email: "john@example.com".to_string(),
            password: "hashed_password".to_string(),
            created_at: now,
            updated_at: Some(now),
            is_active: 1,
        };

        let active_dto = UserReadDto::from(active_user);
        assert_eq!(active_dto.is_active, 1);

        // Test inactive user
        let inactive_user = User {
            id: 2,
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            user_name: "janedoe".to_string(),
            email: "jane@example.com".to_string(),
            password: "hashed_password".to_string(),
            created_at: now,
            updated_at: Some(now),
            is_active: 0,
        };

        let inactive_dto = UserReadDto::from(inactive_user);
        assert_eq!(inactive_dto.is_active, 0);
    }
}

#[cfg(test)]
mod user_register_dto_unit_tests {
    use super::*;
    // UserRegisterDto Tests
    #[test]
    fn test_user_register_dto_valid_creation() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
        };

        assert_eq!(register_dto.email, "test@example.com");
        assert_eq!(register_dto.password, "password123");
        assert_eq!(register_dto.first_name, "John".to_string());
        assert_eq!(register_dto.last_name, "Doe".to_string());
        assert_eq!(register_dto.user_name, "johndoe");
    }

    #[test]
    fn test_user_register_dto_validation_valid() {
        let register_dto = UserRegisterDto {
            email: "valid@example.com".to_string(),
            password: "validpass".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "validuser".to_string(),
        };

        assert!(register_dto.validate().is_ok());
    }

    #[test]
    fn test_user_register_dto_validation_invalid_email() {
        let register_dto = UserRegisterDto {
            email: "invalid-email".to_string(),
            password: "validpass".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "validuser".to_string(),
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("email"));
    }

    #[test]
    fn test_user_register_dto_validation_username_too_short() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "validpass".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "ab".to_string(), // Too short
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("user_name"));
    }

    #[test]
    fn test_user_register_dto_validation_username_too_long() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "validpass".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "a".repeat(21), // Too long
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("user_name"));
    }

    #[test]
    fn test_user_register_dto_debug_format() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "secret123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
        };

        let debug_str = format!("{:?}", register_dto);
        assert!(debug_str.contains("John"));
        assert!(debug_str.contains("Doe"));
        assert!(debug_str.contains("johndoe"));
        assert!(debug_str.contains("test@example.com"));
        assert!(!debug_str.contains("secret123")); // Password should not be in debug output
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
        assert_eq!(register_dto.first_name, cloned_dto.first_name);
        assert_eq!(register_dto.last_name, cloned_dto.last_name);
        assert_eq!(register_dto.user_name, cloned_dto.user_name);
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

        let serialized = serde_json::to_string(&register_dto).unwrap();
        let deserialized: UserRegisterDto = serde_json::from_str(&serialized).unwrap();

        assert_eq!(register_dto.email, deserialized.email);
        assert_eq!(register_dto.first_name, deserialized.first_name);
        assert_eq!(register_dto.last_name, deserialized.last_name);
        assert_eq!(register_dto.user_name, deserialized.user_name);
    }
}

#[cfg(test)]
mod user_read_dto_unit_tests {
    use super::*;
    // UserReadDto Tests
    #[test]
    fn test_user_read_dto_from_user_entity() {
        let now = Utc::now();
        let user = User {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
            email: "john@example.com".to_string(),
            password: "hashed_password".to_string(),
            created_at: now,
            updated_at: Some(now),
            is_active: 1,
        };

        let read_dto = UserReadDto::from(user.clone());

        assert_eq!(read_dto.id, user.id);
        assert_eq!(read_dto.first_name, user.first_name);
        assert_eq!(read_dto.last_name, user.last_name);
        assert_eq!(read_dto.user_name, user.user_name);
        assert_eq!(read_dto.email, user.email);
        assert_eq!(read_dto.created_at, user.created_at);
        assert_eq!(read_dto.updated_at, user.updated_at);
        assert_eq!(read_dto.is_active, user.is_active);
    }

    #[test]
    fn test_user_read_dto_clone() {
        let now = Utc::now();
        let read_dto = UserReadDto {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
            email: "john@example.com".to_string(),
            created_at: now,
            updated_at: Some(now),
            is_active: 1,
        };

        let cloned_dto = read_dto.clone();
        assert_eq!(read_dto.id, cloned_dto.id);
        assert_eq!(read_dto.first_name, cloned_dto.first_name);
        assert_eq!(read_dto.last_name, cloned_dto.last_name);
        assert_eq!(read_dto.user_name, cloned_dto.user_name);
        assert_eq!(read_dto.email, cloned_dto.email);
        assert_eq!(read_dto.created_at, cloned_dto.created_at);
        assert_eq!(read_dto.updated_at, cloned_dto.updated_at);
        assert_eq!(read_dto.is_active, cloned_dto.is_active);
    }

    #[test]
    fn test_user_read_dto_debug_format() {
        let now = Utc::now();
        let read_dto = UserReadDto {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
            email: "john@example.com".to_string(),
            created_at: now,
            updated_at: Some(now),
            is_active: 1,
        };

        let debug_str = format!("{:?}", read_dto);
        assert!(debug_str.contains("John"));
        assert!(debug_str.contains("Doe"));
        assert!(debug_str.contains("johndoe"));
        assert!(debug_str.contains("john@example.com"));
        assert!(debug_str.contains("1"));
    }

    #[test]
    fn test_user_read_dto_serialization() {
        let now = Utc::now();
        let read_dto = UserReadDto {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
            email: "john@example.com".to_string(),
            created_at: now,
            updated_at: Some(now),
            is_active: 1,
        };

        let serialized = serde_json::to_string(&read_dto).unwrap();
        let deserialized: UserReadDto = serde_json::from_str(&serialized).unwrap();

        assert_eq!(read_dto.id, deserialized.id);
        assert_eq!(read_dto.first_name, deserialized.first_name);
        assert_eq!(read_dto.last_name, deserialized.last_name);
        assert_eq!(read_dto.user_name, deserialized.user_name);
        assert_eq!(read_dto.email, deserialized.email);
        assert_eq!(read_dto.created_at, deserialized.created_at);
        assert_eq!(read_dto.updated_at, deserialized.updated_at);
        assert_eq!(read_dto.is_active, deserialized.is_active);
    }

}