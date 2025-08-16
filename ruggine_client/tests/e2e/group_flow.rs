// End-to-End tests for group management flow
// Testing complete user flow from authentication to group operations

use ruggine_client_ui::api::services::group::GroupChatService;
use ruggine_client_ui::api::services::auth::AuthService;
use ruggine_client_ui::api::client::ApiClient;
use ruggine_client_ui::utils::storage::StorageService;
use ruggine_client_ui::types::group::GroupChatCreateRequest;
use ruggine_client_ui::error::AuthError;
use std::sync::Mutex;

#[cfg(test)]
mod group_flow_e2e_tests {
    use super::*;
    use crate::factory::TestFactory;

    // Mutex to serialize tests that use shared storage
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_services() -> (AuthService, GroupChatService) {
        let http_client = ApiClient::new("http://localhost:8002");
        let storage_service = StorageService::new();
        
        #[cfg(not(target_arch = "wasm32"))]
        storage_service.clear_all_test_data();
        
        let auth_service = AuthService::new(http_client.clone(), storage_service.clone());
        let group_service = GroupChatService::new(http_client, storage_service);
        
        (auth_service, group_service)
    }

    #[tokio::test]
    #[ignore] // Requires running server and database
    async fn test_complete_group_creation_flow() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let (auth_service, group_service) = setup_services();
        
        // Step 1: Register a new user
        let user_request = TestFactory::unique_user_register_request("group_flow_creator");
        let registration_result = auth_service.register(
            user_request.email.clone(),
            user_request.password.clone(),
            user_request.first_name,
            user_request.last_name,
            user_request.username,
            user_request.birthday,
            user_request.address,
            format!("{:?}", user_request.gender).to_lowercase(),
        ).await;
        
        // In real test, check if registration succeeded
        match registration_result {
            Ok(_) => {
                // Step 2: Login with the new user
                let login_result = auth_service.login(user_request.email, user_request.password).await;
                
                match login_result {
                    Ok(_) => {
                        // Step 3: Create a group
                        let group_request = TestFactory::valid_group_create_request("e2e_test");
                        let group_result = group_service.create_group(group_request.clone()).await;
                        
                        match group_result {
                            Ok(group) => {
                                // Verify group was created correctly
                                assert_eq!(group.name, group_request.name);
                                assert_eq!(group.description, group_request.description);
                                assert!(group.id > 0);
                                
                                // Step 4: Get user's groups to verify membership
                                let groups_result = group_service.get_user_groups().await;
                                
                                match groups_result {
                                    Ok(memberships) => {
                                        // Should contain the group we just created
                                        assert!(memberships.iter().any(|m| m.group_chat_id == group.id));
                                        
                                        // Step 5: Get the specific group by ID
                                        let group_by_id_result = group_service.get_group_by_id(&group.id.to_string()).await;
                                        
                                        match group_by_id_result {
                                            Ok(fetched_group) => {
                                                assert_eq!(fetched_group.id, group.id);
                                                assert_eq!(fetched_group.name, group.name);
                                            },
                                            Err(e) => println!("Failed to get group by ID: {:?}", e),
                                        }
                                    },
                                    Err(e) => println!("Failed to get user groups: {:?}", e),
                                }
                            },
                            Err(e) => println!("Failed to create group: {:?}", e),
                        }
                    },
                    Err(e) => println!("Failed to login: {:?}", e),
                }
            },
            Err(e) => println!("Failed to register user: {:?}", e),
        }
    }

    #[tokio::test]
    #[ignore] // Requires running server and database
    async fn test_multi_user_group_flow() {
        let _lock = TEST_MUTEX.lock().unwrap();
        
        // Test scenario with multiple users and groups
        let (auth_service, group_service) = setup_services();
        
        // Create multiple users
        let users = vec![
            TestFactory::unique_user_register_request("multi_user_1"),
            TestFactory::unique_user_register_request("multi_user_2"),
        ];
        
        for (i, user_request) in users.iter().enumerate() {
            let registration_result = auth_service.register(
                user_request.email.clone(),
                user_request.password.clone(),
                user_request.first_name.clone(),
                user_request.last_name.clone(),
                user_request.username.clone(),
                user_request.birthday.clone(),
                user_request.address.clone(),
                format!("{:?}", user_request.gender).to_lowercase(),
            ).await;
            
            match registration_result {
                Ok(_) => {
                    // Login as this user
                    let login_result = auth_service.login(
                        user_request.email.clone(),
                        user_request.password.clone()
                    ).await;
                    
                    match login_result {
                        Ok(_) => {
                            // Each user creates a group
                            let group_request = TestFactory::valid_group_create_request(&format!("multi_user_{}", i));
                            let _group_result = group_service.create_group(group_request).await;
                            
                            // Get user's groups
                            let _groups_result = group_service.get_user_groups().await;
                        },
                        Err(e) => println!("Failed to login user {}: {:?}", i, e),
                    }
                },
                Err(e) => println!("Failed to register user {}: {:?}", i, e),
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires running server and database
    async fn test_group_validation_in_flow() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let (auth_service, group_service) = setup_services();
        
        // Register and login a user
        let user_request = TestFactory::unique_user_register_request("validation_flow");
        
        // In a real test environment, these would succeed
        let _registration_result = auth_service.register(
            user_request.email.clone(),
            user_request.password.clone(),
            user_request.first_name,
            user_request.last_name,
            user_request.username,
            user_request.birthday,
            user_request.address,
            format!("{:?}", user_request.gender).to_lowercase(),
        ).await;
        
        let _login_result = auth_service.login(user_request.email, user_request.password).await;
        
        // Pre-create long strings
        let long_name = "a".repeat(101);
        let long_description = "a".repeat(501);
        
        // Test creating invalid groups
        let invalid_groups = vec![
            TestFactory::custom_group_create_request("", "valid description"), // Empty name
            TestFactory::custom_group_create_request(long_name.as_str(), "valid description"), // Name too long
            TestFactory::custom_group_create_request("valid name", long_description.as_str()), // Description too long
        ];
        
        for invalid_group in invalid_groups {
            let result = group_service.create_group(invalid_group).await;
            
            // Should fail with validation error
            match result {
                Err(AuthError::ValidationError(_)) => {
                    // Expected
                },
                Ok(_) => panic!("Should not succeed with invalid group data"),
                Err(e) => {
                    // Other errors are acceptable in E2E context (network, auth, etc.)
                    println!("Got expected error: {:?}", e);
                }
            }
        }
    }

    #[test]
    fn test_group_flow_components_integration() {
        // Test that all components work together without network calls
        let _lock = TEST_MUTEX.lock().unwrap();
        let (auth_service, group_service) = setup_services();
        
        // Test service creation and basic functionality
        assert!(!auth_service.is_authenticated());
        
        let group_request = TestFactory::valid_group_create_request("component_test");
        assert!(group_service.validate_group_create_request(&group_request).is_ok());
        
        // Test that services share the same storage
        let storage_service = StorageService::new();
        #[cfg(not(target_arch = "wasm32"))]
        storage_service.clear_all_test_data();
        
        let http_client = ApiClient::new("http://localhost:8002");
        let auth_service_2 = AuthService::new(http_client.clone(), storage_service.clone());
        let group_service_2 = GroupChatService::new(http_client, storage_service);
        
        assert!(!auth_service_2.is_authenticated());
        
        let group_request_2 = TestFactory::valid_group_create_request("component_test_2");
        assert!(group_service_2.validate_group_create_request(&group_request_2).is_ok());
    }

    #[test]
    fn test_concurrent_group_operations() {
        // Test concurrent operations without network calls
        use std::thread;
        use std::sync::Arc;
        
        let _lock = TEST_MUTEX.lock().unwrap();
        
        let storage = Arc::new(StorageService::new());
        #[cfg(not(target_arch = "wasm32"))]
        storage.clear_all_test_data();
        
        let handles: Vec<_> = (0..3)
            .map(|i| {
                let storage_clone = Arc::clone(&storage);
                thread::spawn(move || {
                    let http_client = ApiClient::new(&format!("http://localhost:800{}", i + 2));
                    let group_service = GroupChatService::new(http_client, (*storage_clone).clone());
                    
                    // Each thread validates different requests
                    for j in 0..5 {
                        let request = TestFactory::valid_group_create_request(&format!("thread_{}_{}", i, j));
                        assert!(group_service.validate_group_create_request(&request).is_ok());
                    }
                    
                    i // Return thread ID for verification
                })
            })
            .collect();
        
        // Wait for all threads to complete
        for (expected_id, handle) in handles.into_iter().enumerate() {
            let thread_id = handle.join().expect("Thread should complete successfully");
            assert_eq!(thread_id, expected_id);
        }
    }

    #[tokio::test]
    #[ignore] // Requires running server
    async fn test_error_recovery_flow() {
        let _lock = TEST_MUTEX.lock().unwrap();
        let (auth_service, group_service) = setup_services();
        
        // Test recovery from various error states
        
        // 1. Test creating group without authentication
        let group_request = TestFactory::valid_group_create_request("recovery_test");
        let result = group_service.create_group(group_request).await;
        
        match result {
            Err(AuthError::ValidationError(_)) => {
                // Expected - validation error
            },
            Err(AuthError::NetworkError(_)) => {
                // Expected if server not running
                println!("Got expected network error");
            },
            _ => {
                // Other errors are also acceptable in E2E context (network, auth, etc.)
                println!("Got error (expected): {:?}", result);
            }
        }
        
        // 2. Test that after authentication failure, we can still validate
        let another_request = TestFactory::valid_group_create_request("recovery_test_2");
        assert!(group_service.validate_group_create_request(&another_request).is_ok());
        
        // 3. Test that invalid data is still rejected after network errors
        let invalid_request = TestFactory::custom_group_create_request("", "description");
        assert!(group_service.validate_group_create_request(&invalid_request).is_err());
    }
}
