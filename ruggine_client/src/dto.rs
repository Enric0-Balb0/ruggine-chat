use serde::{Deserialize, Serialize};

/// Login request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Registration request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
}

/// Password change request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordChangeRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Token response from authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_in: Option<u64>,
}

/// User profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Empty response for endpoints that don't return data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyResponse;
