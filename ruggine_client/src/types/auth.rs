use serde::{Deserialize, Serialize};

/// Login request - exact server DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Token response wrapper from server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSuccessResponseTokenReadDto {
    pub data: serde_json::Value,
}

// =============================================================================
// CLIENT TYPES - Rich types with business logic
// =============================================================================

/// Token response with utility methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub iat: i64,
    pub exp: i64,
}

/// Decoded token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: i32,
    pub email: String,
    pub iat: i64,
    pub exp: i64,
}

// =============================================================================
// CONVERSIONS - From server wrapper types
// =============================================================================

impl From<ApiSuccessResponseTokenReadDto> for TokenResponse {
    fn from(response: ApiSuccessResponseTokenReadDto) -> Self {
        // TODO: Implement conversion when exact server structure is available
        todo!("Implement conversion from ApiSuccessResponseTokenReadDto")
    }
}

// =============================================================================
// IMPLEMENTATIONS - Business logic methods
// =============================================================================

impl TokenResponse {
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now >= self.exp
    }

    /// Get remaining time to expiry in seconds
    pub fn time_to_expiry(&self) -> i64 {
        let now = chrono::Utc::now().timestamp();
        (self.exp - now).max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_creation() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        assert!(!request.email.is_empty());
        assert!(request.email.contains('@'));
        assert!(!request.password.is_empty());
    }

    #[test]
    fn test_token_response_business_logic() {
        let now = chrono::Utc::now().timestamp();
        
        // Valid token
        let valid_token = TokenResponse {
            token: "valid_token".to_string(),
            iat: now,
            exp: now + 3600, // 1 hour from now
        };

        // Expired token
        let expired_token = TokenResponse {
            token: "expired_token".to_string(),
            iat: now - 7200, // 2 hours ago
            exp: now - 3600, // 1 hour ago
        };

        assert!(!valid_token.is_expired());
        assert!(expired_token.is_expired());
        
        assert!(valid_token.time_to_expiry() > 0);
        assert_eq!(expired_token.time_to_expiry(), 0);
    }
}
