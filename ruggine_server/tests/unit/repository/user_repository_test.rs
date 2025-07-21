use ruggine_server::entity::user::{User, NewUser};

#[cfg(test)]
mod user_repository_unit_tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn test_new_user_creation() {
        // Arrange & Act
        let new_user = NewUser {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
            email: "john.doe@example.com".to_string(),
            password: "hashed_password".to_string(),
            is_active: 1, // i8 value (1 for active)
        };

        // Assert
        assert_eq!(new_user.first_name, "John".to_string());
        assert_eq!(new_user.last_name, "Doe".to_string());
        assert_eq!(new_user.user_name, "johndoe");
        assert_eq!(new_user.email, "john.doe@example.com");
        assert_eq!(new_user.password, "hashed_password");
        assert_eq!(new_user.is_active, 1);
    }

    #[test]
    fn test_user_validation() {
        // Test to validate user data
        let valid_email = "test@example.com";
        let invalid_email = "invalid-email";

        assert!(valid_email.contains('@'));
        assert!(!invalid_email.contains('@'));
    }

    // Test to verify data structure
    #[test]
    fn test_user_struct_fields() {
        use chrono::Utc;
        
        let now = Utc::now();
        let user = User {
            id: 1,
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            user_name: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password".to_string(),
            is_active: 1, // i8 value (1 for active)
            created_at: now,
            updated_at: None,
        };

        assert_eq!(user.id, 1);
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.is_active, 1);
        assert_eq!(user.first_name, "Test".to_string());
        assert_eq!(user.updated_at, None);
    }

    #[test]
    fn test_user_creation() {
        // Test creating a User struct 
        let now = Utc::now();
        let user = User {
            id: 1,
            first_name: "Jane".to_string(),
            last_name: "Smith".to_string(),
            user_name: "janesmith".to_string(),
            email: "jane@example.com".to_string(),
            password: "hashed_password".to_string(),
            created_at: now,
            updated_at: None,
            is_active: 1,
        };

        assert_eq!(user.id, 1);
        assert_eq!(user.user_name, "janesmith");
        assert_eq!(user.email, "jane@example.com");
        assert_eq!(user.first_name, "Jane".to_string());
        assert_eq!(user.is_active, 1);
    }

    #[test]
    fn test_inactive_user() {
        // Test inactive user
        let new_user = NewUser {
            first_name: "Inactive".to_string(),
            last_name: "User".to_string(),
            user_name: "inactive".to_string(),
            email: "inactive@example.com".to_string(),
            password: "password".to_string(),
            is_active: 0, // 0 for inactive
        };

        assert_eq!(new_user.is_active, 0);
    }

    #[test]
    fn test_user_with_none_updated_at() {
        // Test User with None updated_at (common case for new users)
        let now = Utc::now();
        let user = User {
            id: 42,
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            user_name: "testuser42".to_string(),
            email: "test42@example.com".to_string(),
            password: "secure_hash".to_string(),
            created_at: now,
            updated_at: None, // User has never been updated
            is_active: 1,
        };

        assert_eq!(user.id, 42);
        assert_eq!(user.updated_at, None);
        assert!(user.created_at <= Utc::now()); // Created time should be in the past or now
    }

    #[test]
    fn test_user_clone_functionality() {
        // Test that User structs can be cloned (important for repositories)
        let original_user = NewUser {
            first_name: "Original".to_string(),
            last_name: "User".to_string(),
            user_name: "original".to_string(),
            email: "original@example.com".to_string(),
            password: "password".to_string(),
            is_active: 1,
        };

        let cloned_user = original_user.clone();

        assert_eq!(original_user.user_name, cloned_user.user_name);
        assert_eq!(original_user.email, cloned_user.email);
        assert_eq!(original_user.first_name, cloned_user.first_name);
    }

    #[test]
    fn test_edge_case_email_formats() {
        // Test various email formats that should be valid
        let test_cases = vec![
            "simple@example.com",
            "test.email@example.com", 
            "user+tag@example.com",
            "user123@sub.example.com",
        ];

        for email in test_cases {
            let new_user = NewUser {
                first_name: "Test".to_string(),
                last_name: "User".to_string(),
                user_name: format!("user_{}", email.replace("@", "_at_").replace(".", "_")),
                email: email.to_string(),
                password: "password".to_string(),
                is_active: 1,
            };

            assert_eq!(new_user.email, email);
            assert!(!new_user.user_name.is_empty());
        }
    }

    #[test]
    fn test_inactive_new_user() {
        // Test creating an inactive user
        let new_user = NewUser {
            first_name: "Inactive".to_string(),
            last_name: "User".to_string(),
            user_name: "inactive_user".to_string(),
            email: "inactive@example.com".to_string(),
            password: "password".to_string(),
            is_active: 0, // 0 for inactive
        };

        assert_eq!(new_user.is_active, 0);
        assert_eq!(new_user.first_name, "Inactive".to_string());
    }

    #[test]
    fn test_new_user_validation_fields() {
        // Test that NewUser has all required fields for database insertion
        let new_user = NewUser {
            first_name: "Valid".to_string(),
            last_name: "User".to_string(),
            user_name: "validuser".to_string(),
            email: "valid@example.com".to_string(),
            password: "hashed_password_123".to_string(),
            is_active: 1,
        };

        // Verify all required fields are present
        assert!(!new_user.user_name.is_empty(), "Username should not be empty");
        assert!(!new_user.email.is_empty(), "Email should not be empty");
        assert!(!new_user.password.is_empty(), "Password should not be empty");
        assert!(new_user.email.contains("@"), "Email should contain @ symbol");
        
        // Verify is_active is valid (0 or 1)
        assert!(new_user.is_active == 0 || new_user.is_active == 1, "is_active should be 0 or 1");
    }

    #[test]
    fn test_user_id_types() {
        // Test that User ID is i32 as expected by database
        let user = User {
            id: i32::MAX,
            first_name: "Max".to_string(),
            last_name: "Id".to_string(),
            user_name: "maxid".to_string(),
            email: "max@example.com".to_string(),
            password: "password".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: None,
            is_active: 1,
        };

        assert_eq!(user.id, i32::MAX);
        
        // Test minimum valid ID
        let user_min = User {
            id: 1,
            first_name: "Min".to_string(),
            last_name: "Id".to_string(),
            user_name: "minid".to_string(),
            email: "min@example.com".to_string(),
            password: "password".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: None,
            is_active: 1,
        };

        assert_eq!(user_min.id, 1);
    }

    #[test]
    fn test_user_timestamp_behavior() {
        // Test timestamp handling
        use chrono::Utc;
        
        let now = Utc::now();
        let later = now + chrono::Duration::minutes(5);
        
        let user = User {
            id: 100,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "timestamp_test".to_string(),
            email: "timestamp@example.com".to_string(),
            password: "password".to_string(),
            created_at: now,
            updated_at: Some(later),
            is_active: 1,
        };

        // Verify timestamps are correctly set
        assert_eq!(user.created_at, now);
        assert_eq!(user.updated_at, Some(later));
        assert!(user.updated_at.unwrap() > user.created_at);
    }

    #[test]
    fn test_is_active_states() {
        // Test both active states
        let active_user = NewUser {
            first_name: "Active".to_string(),
            last_name: "User".to_string(),
            user_name: "active_user".to_string(),
            email: "active@example.com".to_string(),
            password: "password".to_string(),
            is_active: 1,
        };

        let inactive_user = NewUser {
            first_name: "Inactive".to_string(),
            last_name: "User".to_string(),
            user_name: "inactive_user".to_string(),
            email: "inactive@example.com".to_string(),
            password: "password".to_string(),
            is_active: 0,
        };

        // Test that active user is marked correctly
        assert_eq!(active_user.is_active, 1);
        assert_eq!(inactive_user.is_active, 0);
        
        // Test boolean-like behavior
        assert!(active_user.is_active != 0);
        assert!(inactive_user.is_active == 0);
    }

    #[test]
    fn test_struct_serialization_compatibility() {
        // Test that structs are compatible with serialization (important for API)
        use serde_json;
        
        let new_user = NewUser {
            first_name: "Serialize".to_string(),
            last_name: "Test".to_string(),
            user_name: "serialize_test".to_string(),
            email: "serialize@example.com".to_string(),
            password: "password123".to_string(),
            is_active: 1,
        };

        // Test that NewUser can be serialized to JSON
        let json_result = serde_json::to_string(&new_user);
        assert!(json_result.is_ok(), "NewUser should be serializable to JSON");
        
        let json_string = json_result.unwrap();
        assert!(json_string.contains("serialize_test"));
        assert!(json_string.contains("serialize@example.com"));

        // Test that it can be deserialized back
        let deserialized_result: Result<NewUser, _> = serde_json::from_str(&json_string);
        assert!(deserialized_result.is_ok(), "NewUser should be deserializable from JSON");
        
        let deserialized_user = deserialized_result.unwrap();
        assert_eq!(deserialized_user.user_name, new_user.user_name);
        assert_eq!(deserialized_user.email, new_user.email);
    }

}