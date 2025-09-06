// Authentication-related test data factory

use super::base_factory::BaseFactory;
use ruggine_client_ui::types::auth::{LoginRequest, TokenResponse};

pub struct AuthFactory;

impl AuthFactory {
    /// Create unique LoginRequest for testing
    pub fn unique_login_request(prefix: &str) -> LoginRequest {
        let (email, _, _) = BaseFactory::get_unique_user_info(prefix);
        
        LoginRequest {
            email,
            password: "testpass123".to_string(),
        }
    }
    
    /// Create basic login request
    pub fn basic_login_request() -> LoginRequest {
        LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        }
    }
    
    /// Create mock token response for testing
    pub fn mock_token_response() -> TokenResponse {
        let unique_token = BaseFactory::get_unique_token();
        
        TokenResponse {
            token: unique_token,
            iat: 1000000000,
            exp: 2000000000,
        }
    }
    
    /// Create expired token response
    pub fn expired_token_response() -> TokenResponse {
        TokenResponse {
            token: "expired_token".to_string(),
            iat: 1000000000,
            exp: 1000000000, // Expired (same time)
        }
    }
    
    /// Create invalid token response
    pub fn invalid_token_response() -> TokenResponse {
        TokenResponse {
            token: "invalid_token".to_string(),
            iat: 1000000000,
            exp: 500000000, // Past expiry
        }
    }
}
