// Common testing utilities for ruggine_client
// Following same patterns as ruggine_server

pub mod factory;

use std::sync::Once;
use wasm_bindgen_test::*;

// Re-export factory for convenience
pub use factory::TestFactory;

static INIT_LOG: Once = Once::new();

/// Initialize test logging (following server pattern)
pub fn init_test_logging() {
    INIT_LOG.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
    });
}

/// Configure wasm-bindgen-test for browser tests
pub fn configure_wasm_tests() {
    wasm_bindgen_test_configure!(run_in_browser);
}

// Simple mock helpers for basic testing
// For more complex scenarios, we'll use mockito directly in tests

pub const MOCK_SERVER_URL: &str = "http://localhost:3000";
pub const MOCK_JWT_TOKEN: &str = "mock_jwt_token_12345";

pub fn mock_user_profile_json() -> &'static str {
    r#"{
        "data": {
            "id": 1,
            "email": "test@example.com",
            "first_name": "Test",
            "last_name": "User",
            "username": "testuser",
            "user_status": "Active",
            "user_type": "User",
            "current_action": "Online",
            "birthday": "1990-01-01",
            "address": "123 Test St",
            "gender": "male",
            "created_at": "2025-01-01T00:00:00Z",
            "updated_at": "2025-01-01T00:00:00Z",
            "last_login": "2025-01-01T00:00:00Z"
        },
        "success": true
    }"#
}

pub fn mock_token_response_json() -> &'static str {
    r#"{
        "data": {
            "token": "mock_jwt_token_12345",
            "iat": 1640995200,
            "exp": 1641081600
        },
        "success": true
    }"#
}
