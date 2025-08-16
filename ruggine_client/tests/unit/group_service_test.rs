// Unit tests for GroupChatService
// Testing group chat management logic, validation, and error handling

use ruggine_client_ui::api::services::group::{GroupChatService, GroupUpdateRequest};
use ruggine_client_ui::utils::storage::StorageService;
use ruggine_client_ui::api::client::ApiClient;
use ruggine_client_ui::error::AuthError;
use ruggine_client_ui::types::group::GroupChatCreateRequest;

#[cfg(test)]
mod group_service_unit_tests {
    use super::*;
    use std::sync::Mutex;
    use crate::factory::TestFactory;

    // Mutex to serialize tests that use shared storage
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_group_service() -> GroupChatService {
        let http_client = ApiClient::new("http://localhost:3000");
        let storage_service = StorageService::new();
        
        // Clear any previous test data
        #[cfg(not(target_arch = "wasm32"))]
        storage_service.clear_all_test_data();
        
        GroupChatService::new(http_client, storage_service)
    }

    #[test]
    fn test_validate_group_create_request_valid() {
        let service = setup_group_service();
        let request = TestFactory::valid_group_create_request("test");
        
        let result = service.validate_group_create_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_group_create_request_empty_name() {
        let service = setup_group_service();
        let mut request = TestFactory::valid_group_create_request("test");
        request.name = "".to_string();
        
        let result = service.validate_group_create_request(&request);
        
        match result {
            Err(AuthError::ValidationError(msg)) => {
                assert!(msg.contains("Group name cannot be empty"));
            },
            _ => panic!("Should fail with empty name validation error"),
        }
    }

    #[test]
    fn test_validate_group_create_request_whitespace_only_name() {
        let service = setup_group_service();
        let mut request = TestFactory::valid_group_create_request("test");
        request.name = "   ".to_string();
        
        let result = service.validate_group_create_request(&request);
        
        match result {
            Err(AuthError::ValidationError(msg)) => {
                assert!(msg.contains("Group name cannot be empty"));
            },
            _ => panic!("Should fail with whitespace name validation error"),
        }
    }

    #[test]
    fn test_validate_group_create_request_name_too_long() {
        let service = setup_group_service();
        let mut request = TestFactory::valid_group_create_request("test");
        request.name = "a".repeat(101); // 101 characters, exceeds limit of 100
        
        let result = service.validate_group_create_request(&request);
        
        match result {
            Err(AuthError::ValidationError(msg)) => {
                assert!(msg.contains("Group name cannot exceed 100 characters"));
            },
            _ => panic!("Should fail with name length validation error"),
        }
    }

    #[test]
    fn test_validate_group_create_request_name_exactly_100_chars() {
        let service = setup_group_service();
        let mut request = TestFactory::valid_group_create_request("test");
        request.name = "a".repeat(100); // Exactly 100 characters, should be valid
        
        let result = service.validate_group_create_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_group_create_request_description_too_long() {
        let service = setup_group_service();
        let mut request = TestFactory::valid_group_create_request("test");
        request.description = "a".repeat(501); // 501 characters, exceeds limit of 500
        
        let result = service.validate_group_create_request(&request);
        
        match result {
            Err(AuthError::ValidationError(msg)) => {
                assert!(msg.contains("Group description cannot exceed 500 characters"));
            },
            _ => panic!("Should fail with description length validation error"),
        }
    }

    #[test]
    fn test_validate_group_create_request_description_exactly_500_chars() {
        let service = setup_group_service();
        let mut request = TestFactory::valid_group_create_request("test");
        request.description = "a".repeat(500); // Exactly 500 characters, should be valid
        
        let result = service.validate_group_create_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_group_create_request_empty_description_valid() {
        let service = setup_group_service();
        let mut request = TestFactory::valid_group_create_request("test");
        request.description = "".to_string(); // Empty description should be valid
        
        let result = service.validate_group_create_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_group_with_network_error() {
        let service = setup_group_service();
        let request = TestFactory::valid_group_create_request("test");
        
        // This should fail with network error since we're using localhost:3000
        let result = tokio_test::block_on(async {
            service.create_group(request).await
        });
        
        // Should not fail with validation error (will fail with network)
        match result {
            Err(AuthError::ValidationError(_)) => panic!("Should not fail validation"),
            Err(_) => {}, // Expected - network error
            Ok(_) => panic!("Unexpected success without server"),
        }
    }

    #[test]
    fn test_get_user_groups_with_network_error() {
        let service = setup_group_service();
        
        let result = tokio_test::block_on(async {
            service.get_user_groups().await
        });
        
        // Should fail with network error
        match result {
            Err(_) => {}, // Expected - network error
            Ok(_) => panic!("Unexpected success without server"),
        }
    }

    #[test]
    fn test_get_group_by_id_with_network_error() {
        let service = setup_group_service();
        
        let result = tokio_test::block_on(async {
            service.get_group_by_id("1").await
        });
        
        // Should fail with network error
        match result {
            Err(_) => {}, // Expected - network error
            Ok(_) => panic!("Unexpected success without server"),
        }
    }

    #[test]
    fn test_update_group_with_network_error() {
        let service = setup_group_service();
        let update_request = GroupUpdateRequest {
            name: Some("Updated Name".to_string()),
            description: Some("Updated Description".to_string()),
        };
        
        let result = tokio_test::block_on(async {
            service.update_group("1", update_request).await
        });
        
        // Should fail with network error
        match result {
            Err(_) => {}, // Expected - network error
            Ok(_) => panic!("Unexpected success without server"),
        }
    }

    #[test]
    fn test_delete_group_with_network_error() {
        let service = setup_group_service();
        
        let result = tokio_test::block_on(async {
            service.delete_group("1").await
        });
        
        // Should fail with network error
        match result {
            Err(_) => {}, // Expected - network error
            Ok(_) => panic!("Unexpected success without server"),
        }
    }

    #[test]
    fn test_default_group_service_creation() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let service = GroupChatService::default();
        
        // Should be able to create default service without errors
        let request = TestFactory::valid_group_create_request("default");
        let result = service.validate_group_create_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_group_validation_edge_cases() {
        let service = setup_group_service();
        
        // Pre-create the long strings to avoid borrowing issues
        let long_description = "a".repeat(501);
        let long_name = "a".repeat(101);
        
        // Test with various edge cases
        let test_cases = vec![
            // (name, description, should_pass, error_substring)
            ("a", "", true, ""), // Minimal valid name
            ("Valid Group Name", "Valid description", true, ""), // Normal case
            ("Group\n\tName", "Description with\nnewlines", true, ""), // With whitespace chars
            ("", "Valid description", false, "Group name cannot be empty"), // Empty name
            ("Valid Name", long_description.as_str(), false, "Group description cannot exceed 500 characters"), // Long description
            (long_name.as_str(), "Valid description", false, "Group name cannot exceed 100 characters"), // Long name
        ];
        
        for (name, description, should_pass, expected_error) in test_cases {
            let request = GroupChatCreateRequest {
                name: name.to_string(),
                description: description.to_string(),
            };
            
            let result = service.validate_group_create_request(&request);
            
            if should_pass {
                assert!(result.is_ok(), "Should pass validation for name: '{}', description length: {}", name, description.len());
            } else {
                match result {
                    Err(AuthError::ValidationError(msg)) => {
                        assert!(msg.contains(expected_error), "Expected error '{}' but got '{}'", expected_error, msg);
                    },
                    _ => panic!("Should fail with validation error for name: '{}', description length: {}", name, description.len()),
                }
            }
        }
    }

    #[test]
    fn test_group_update_request_serialization() {
        // Test that GroupUpdateRequest can be serialized/deserialized correctly
        let update_request = GroupUpdateRequest {
            name: Some("New Name".to_string()),
            description: Some("New Description".to_string()),
        };
        
        let serialized = serde_json::to_string(&update_request);
        assert!(serialized.is_ok());
        
        let deserialized: Result<GroupUpdateRequest, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
        
        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized.name, Some("New Name".to_string()));
        assert_eq!(deserialized.description, Some("New Description".to_string()));
    }

    #[test]
    fn test_group_update_request_partial() {
        // Test partial updates
        let name_only = GroupUpdateRequest {
            name: Some("New Name".to_string()),
            description: None,
        };
        
        let description_only = GroupUpdateRequest {
            name: None,
            description: Some("New Description".to_string()),
        };
        
        let both_none = GroupUpdateRequest {
            name: None,
            description: None,
        };
        
        // All should serialize correctly
        assert!(serde_json::to_string(&name_only).is_ok());
        assert!(serde_json::to_string(&description_only).is_ok());
        assert!(serde_json::to_string(&both_none).is_ok());
    }
}
