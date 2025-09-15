// Integration tests for GroupChatService
// Testing integration between GroupChatService and external dependencies

use ruggine_client_ui::api::services::group::{GroupChatService, GroupUpdateRequest};
use ruggine_client_ui::api::client::ApiClient;
use ruggine_client_ui::utils::storage::StorageService;
use ruggine_client_ui::error::AuthError;
use ruggine_client_ui::types::group::GroupChatCreateRequest;
use std::sync::Mutex;

#[cfg(test)]
mod group_service_integration_tests {
    use super::*;
    use crate::common::TestFactory;

    // Mutex to serialize tests that use shared storage
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_service_with_mock_server() -> GroupChatService {
        // Use a mock server URL that we can control in integration tests
        let http_client = ApiClient::new("http://localhost:8002"); // Matches server port
        let storage_service = StorageService::new();
        
        #[cfg(not(target_arch = "wasm32"))]
        storage_service.clear_all_test_data();
        
        GroupChatService::new(http_client, storage_service)
    }

    // Note: These tests require a running server instance for full integration testing
    // In a CI/CD environment, you would start a test server before running these tests

    #[tokio::test]
    #[ignore] // Ignored by default, run with --ignored when server is available
    async fn test_create_group_integration_with_server() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let service = setup_service_with_mock_server();
        
        // First, we need to authenticate to get a valid token
        // This would typically be done through AuthService in a real scenario
        // For now, we'll test that the service properly handles the request structure
        
        let request = TestFactory::valid_group_create_request("integration");
        
        let result = service.create_group(request).await;
        
        // Without authentication, we expect an unauthorized error
        match result {
            Err(AuthError::ValidationError(_)) => {
                // Validation error is also acceptable 
            },
            Err(AuthError::NetworkError(_)) => {
                // Expected - if server is not running
            },
            Ok(_) => panic!("Should not succeed without authentication"),
            Err(e) => println!("Got error (expected): {:?}", e), // Other errors are also acceptable for integration test
        }
    }

    #[tokio::test]
    #[ignore] // Ignored by default, run with --ignored when server is available
    async fn test_get_user_groups_integration_with_server() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let service = setup_service_with_mock_server();
        
        let result = service.get_user_groups().await;
        
        // Without authentication, we expect an unauthorized error
        match result {
            Err(AuthError::ValidationError(_)) => {
                // Validation error is also acceptable
            },
            Err(AuthError::NetworkError(_)) => {
                // Expected - if server is not running
            },
            Ok(_) => panic!("Should not succeed without authentication"),
            Err(e) => println!("Got error (expected): {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore] // Ignored by default, run with --ignored when server is available  
    async fn test_get_group_by_id_integration_with_server() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let service = setup_service_with_mock_server();
        
        let result = service.get_group_by_id("1").await;
        
        // Without authentication, we expect an unauthorized error
        match result {
            Err(AuthError::ValidationError(_)) => {
                // Validation error is also acceptable
            },
            Err(AuthError::NetworkError(_)) => {
                // Expected - if server is not running
            },
            Ok(_) => panic!("Should not succeed without authentication"),
            Err(e) => println!("Got error (expected): {:?}", e),
        }
    }

    #[test]
    fn test_service_integration_with_storage() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let storage_service = StorageService::new();
        let http_client = ApiClient::new("http://localhost:8002");
        
        #[cfg(not(target_arch = "wasm32"))]
        storage_service.clear_all_test_data();
        
        // Test that service can be created with real storage
        let service = GroupChatService::new(http_client, storage_service.clone());
        
        // Test validation still works with real storage
        let valid_request = TestFactory::valid_group_create_request("storage_test");
        assert!(service.validate_group_create_request(&valid_request).is_ok());
        
        let invalid_request = TestFactory::custom_group_create_request("", "description");
        assert!(service.validate_group_create_request(&invalid_request).is_err());
    }

    #[test]  
    fn test_service_with_different_base_urls() {
        // Test that service can handle different server URLs
        let test_urls = vec![
            "http://localhost:8002",
            "https://api.example.com",
            "http://127.0.0.1:3000",
        ];
        
        for url in test_urls {
            let http_client = ApiClient::new(url);
            let storage_service = StorageService::new();
            
            let service = GroupChatService::new(http_client, storage_service);
            
            // Basic validation should work regardless of URL
            let request = TestFactory::valid_group_create_request("url_test");
            assert!(service.validate_group_create_request(&request).is_ok());
        }
    }

    #[test]
    fn test_service_serialization_compatibility() {
        // Test that our request/response types are compatible with expected JSON format
        let service = setup_service_with_mock_server();
        
        // Test GroupChatCreateRequest serialization
        let create_request = TestFactory::valid_group_create_request("serialization");
        let json_result = serde_json::to_string(&create_request);
        assert!(json_result.is_ok());
        
        let json = json_result.unwrap();
        assert!(json.contains("name"));
        assert!(json.contains("description"));
        
        // Test deserialization back
        let deserialized: Result<GroupChatCreateRequest, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok());
        
        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized.name, create_request.name);
        assert_eq!(deserialized.description, create_request.description);
        
        // Test GroupUpdateRequest serialization
        let update_request = GroupUpdateRequest {
            name: Some("Updated Name".to_string()),
            description: Some("Updated Description".to_string()),
        };
        
        let json_result = serde_json::to_string(&update_request);
        assert!(json_result.is_ok());
        
        let json = json_result.unwrap();
        assert!(json.contains("name"));
        assert!(json.contains("description"));
    }

    #[test]
    fn test_error_handling_integration() {
        // Test different error scenarios in integration context
        let service = setup_service_with_mock_server();
        
        // Pre-create long strings
        let long_name = "a".repeat(101);
        let long_description = "a".repeat(501);
        
        // Test validation errors are properly propagated
        let invalid_requests = vec![
            TestFactory::custom_group_create_request("", "valid description"),
            TestFactory::custom_group_create_request(long_name.as_str(), "valid description"),
            TestFactory::custom_group_create_request("valid name", long_description.as_str()),
        ];
        
        for request in invalid_requests {
            let result = service.validate_group_create_request(&request);
            match result {
                Err(AuthError::ValidationError(_)) => {
                    // Expected validation error
                },
                _ => panic!("Should return validation error"),
            }
        }
    }

    #[test]
    fn test_concurrent_service_creation() {
        use std::thread;
        
        // Test that multiple services can be created concurrently without issues
        let handles: Vec<_> = (0..5)
            .map(|i| {
                thread::spawn(move || {
                    let http_client = ApiClient::new(&format!("http://localhost:800{}", i));
                    let storage_service = StorageService::new();
                    let service = GroupChatService::new(http_client, storage_service);
                    
                    // Test basic functionality
                    let request = TestFactory::valid_group_create_request(&format!("concurrent_{}", i));
                    service.validate_group_create_request(&request)
                })
            })
            .collect();
        
        // Wait for all threads to complete
        for handle in handles {
            let result = handle.join().expect("Thread should complete successfully");
            assert!(result.is_ok());
        }
    }
}

// Helper struct for testing with mock responses (could be extended for more sophisticated testing)
#[allow(dead_code)]
struct MockServer {
    port: u16,
}

#[allow(dead_code)]
impl MockServer {
    fn new(port: u16) -> Self {
        Self { port }
    }
    
    fn url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }
}

// Integration test helpers
mod integration_helpers {
    #[allow(dead_code)]
    pub fn is_server_running(url: &str) -> bool {
        // Simple check to see if server is accessible
        // In real integration tests, this would be more sophisticated
        use std::time::Duration;
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build();
            
        if let Ok(client) = client {
            matches!(
                tokio_test::block_on(async {
                    client.get(url).send().await
                }),
                Ok(_)
            )
        } else {
            false
        }
    }
    
    #[allow(dead_code)]
    pub fn wait_for_server(url: &str, max_attempts: u32) -> bool {
        use std::thread::sleep;
        use std::time::Duration;
        
        for _ in 0..max_attempts {
            if is_server_running(url) {
                return true;
            }
            sleep(Duration::from_millis(100));
        }
        false
    }
}
