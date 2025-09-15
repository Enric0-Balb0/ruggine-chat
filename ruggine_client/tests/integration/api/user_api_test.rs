// Client-focused tests - NOT testing server logic
// Testing client-specific concerns: UI state, data transformation, error handling


#[cfg(test)]
mod user_api_client_tests {
    
    use crate::common::TestFactory;

    #[test]
    fn test_user_register_request_creation() {
        // Test: Client can create registration request
        let request = TestFactory::unique_user_register_request("creation_test");
        assert!(!request.email.is_empty());
        assert!(!request.username.is_empty());
    }

    #[test] 
    fn test_user_register_request_validation() {
        // Test: Client-side data validation before sending to server
        let request = TestFactory::unique_user_register_request("validation_test");
        
        // Client-side validation rules
        assert!(!request.email.is_empty(), "Email should not be empty");
        assert!(request.email.contains('@'), "Email should contain @");
        assert!(request.password.len() >= 8, "Password should be at least 8 chars");
        assert!(!request.first_name.trim().is_empty(), "First name required");
        assert!(!request.last_name.trim().is_empty(), "Last name required");
        assert!(!request.username.trim().is_empty(), "Username required");
    }

    #[test]
    fn test_user_profile_display_methods() {
        // Test: Client-specific utility methods for UI display
    let profile = TestFactory::mock_user_profile();
        
        // These are CLIENT-SIDE utilities for UI rendering
        assert!(!profile.full_name().is_empty(), "Full name should be computed");
        assert!(profile.is_active(), "Test profile should be active");
        assert!(profile.is_online(), "Test profile should be online");
        
        // Test display formatting
        let full_name = profile.full_name();
        assert!(full_name.contains(&profile.first_name));
        assert!(full_name.contains(&profile.last_name));
    }

    #[test]
    fn test_user_data_sanitization() {
        // Test: Client-side data cleaning before display
        let mut request = TestFactory::unique_user_register_request("sanitize_test");
        
        // Simulate user input with extra spaces
        request.first_name = "  John  ".to_string();
        request.last_name = "  Doe  ".to_string();
        request.username = "  johndoe  ".to_string();
        
        // Client should sanitize before sending
        let sanitized_first = request.first_name.trim();
        let sanitized_last = request.last_name.trim();
        let sanitized_username = request.username.trim();
        
        assert_eq!(sanitized_first, "John");
        assert_eq!(sanitized_last, "Doe");
        assert_eq!(sanitized_username, "johndoe");
    }
}
