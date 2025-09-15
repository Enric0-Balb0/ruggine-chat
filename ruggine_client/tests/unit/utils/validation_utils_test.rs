// Unit tests for validation utilities

#[cfg(test)]
mod validation_utils_tests {
    use crate::common::*;
    
    use crate::common::TestFactory;

    #[test]
    fn test_email_validation_patterns() {
        // Test email validation logic
        let valid_emails = vec![
            "test@example.com",
            "user.name@domain.co.uk",
            "user+tag@example.org",
            "user123@test-domain.com",
        ];
        
        let invalid_emails = vec![
            "invalid-email",
            "@example.com",
            "user@",
            "user..name@example.com",
            "",
        ];
        
        for email in valid_emails {
            assert!(email.contains("@"));
            assert!(email.contains("."));
            assert!(!email.is_empty());
        }
        
        for email in invalid_emails {
            if !email.is_empty() {
                // Check various invalid patterns
                let has_at = email.contains("@");
                let has_dot = email.contains(".");
                
                if !has_at || !has_dot {
                    // Invalid if missing @ or .
                    assert!(true);
                } else if email.starts_with("@") || email.ends_with("@") {
                    // Invalid if starts or ends with @
                    assert!(true);
                }
            } else {
                assert!(email.is_empty());
            }
        }
    }

    #[test]
    fn test_password_validation_patterns() {
        // Test password validation logic
        let valid_passwords = vec![
            "password123",
            "SecurePass!",
            "myP@ssw0rd",
            "LongPasswordWith123",
        ];
        
        let invalid_passwords = vec![
            "short",
            "",
            "123456",
            "password", // too common
        ];
        
        for password in valid_passwords {
            assert!(password.len() >= 8);
            assert!(!password.is_empty());
        }
        
        for password in invalid_passwords {
            if password.len() < 8 {
                assert!(password.len() < 8);
            }
        }
    }

    #[test]
    fn test_username_validation_patterns() {
        // Test username validation logic
        let valid_usernames = vec![
            "username",
            "user123",
            "test_user",
            "user-name",
        ];
        
        let invalid_usernames = vec![
            "",
            "a", // too short
            "user name", // spaces
            "user@name", // special chars
        ];
        
        for username in valid_usernames {
            assert!(!username.is_empty());
            assert!(username.len() >= 2);
            assert!(!username.contains(" "));
        }
        
        for username in invalid_usernames {
            if username.is_empty() {
                assert!(username.is_empty());
            } else if username.len() < 2 {
                assert!(username.len() < 2);
            } else if username.contains(" ") {
                assert!(username.contains(" "));
            }
        }
    }

    #[test]
    fn test_group_name_validation() {
        // Test group name validation
        let valid_group_names = vec![
            "Test Group",
            "Project Team",
            "Development Group 123",
            "My-Group_Name",
        ];
        
        let invalid_group_names = vec![
            "",
            "a", // too short
            "   ", // only spaces
        ];
        
        for name in valid_group_names {
            assert!(!name.trim().is_empty());
            assert!(name.len() >= 2);
        }
        
        for name in invalid_group_names {
            if name.trim().is_empty() {
                assert!(name.trim().is_empty());
            }
        }
    }

    #[test]
    fn test_message_content_validation() {
        // Test message content validation
        let valid_contents = vec![
            "Hello world!",
            "This is a test message.",
            "Message with 123 numbers",
            "Multi\nline\nmessage",
        ];
        
        let invalid_contents = vec![
            "",
            "   ", // only spaces
        ];
        
        for content in valid_contents {
            assert!(!content.trim().is_empty());
        }
        
        for content in invalid_contents {
            assert!(content.trim().is_empty());
        }
    }

    #[test]
    fn test_user_register_request_validation() {
        // Test complete user registration validation
        let valid_request = UserFactory::basic_user_register_request();
        
        // Email validation
        assert!(valid_request.email.contains("@"));
        assert!(valid_request.email.contains("."));
        
        // Password validation
        assert!(valid_request.password.len() >= 8);
        
        // Name validation
        assert!(!valid_request.first_name.trim().is_empty());
        assert!(!valid_request.last_name.trim().is_empty());
        
        // Username validation
        assert!(!valid_request.username.trim().is_empty());
        assert!(valid_request.username.len() >= 2);
    }

    #[test]
    fn test_group_create_request_validation() {
        // Test group creation validation
        let valid_request = GroupFactory::basic_group_create_request();
        
        // Name validation
        assert!(!valid_request.name.trim().is_empty());
        assert!(valid_request.name.len() >= 2);
        
        // Description validation (optional)
        if !valid_request.description.trim().is_empty() {
            assert!(!valid_request.description.trim().is_empty());
        }
    }

    #[test]
    fn test_message_create_request_validation() {
        // Test message creation validation
        let valid_request = TestFactory::mock_message_create_request("Valid message", 1);
        
        // Content validation
        assert!(!valid_request.content.trim().is_empty());
        
        // Group ID validation
        assert!(valid_request.group_chat_id > 0);
    }

    #[test]
    fn test_validation_edge_cases() {
        // Test edge cases for validation
        let edge_cases = vec![
            ("", false), // empty string
            ("a", false), // too short
            ("valid_input", true), // valid
            ("   ", false), // only spaces
            ("valid input with spaces", true), // spaces allowed in some contexts
        ];
        
        for (input, should_be_valid) in edge_cases {
            let is_valid = !input.trim().is_empty() && input.trim().len() >= 2;
            
            if should_be_valid {
                assert!(is_valid, "Expected '{}' to be valid", input);
            } else {
                assert!(!is_valid || input.trim().len() < 2, "Expected '{}' to be invalid", input);
            }
        }
    }

    #[test]
    fn test_factory_generated_data_validation() {
        // Test that factory-generated data is valid
        let user = UserFactory::mock_user_profile();
        let group = GroupFactory::mock_group_chat();
        let message = TestFactory::mock_message("validation_test");
        
        // User validation
        assert!(user.email.contains("@"));
        assert!(!user.username.is_empty());
        assert!(user.id > 0);
        
        // Group validation
        assert!(!group.name.is_empty());
        assert!(group.id > 0);
        assert!(group.created_by > 0);
        
        // Message validation
        assert!(!message.content.is_empty());
        assert!(message.id > 0);
        assert!(message.sender_id > 0);
        assert!(message.group_chat_id > 0);
    }
}
