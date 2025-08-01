use crate::http::{client::ApiClient, error::HttpError};
use crate::services::storage_service::StorageService;
use crate::error::AuthError;
use crate::dto::{UserProfile, TokenResponse};
use crate::config::{constants::{AuthConstants, AppConstants}, endpoints::ApiEndpoints};

// Direct type imports (organizzati per modulo)
use crate::types::auth::{LoginRequest, TokenResponse as TypesTokenResponse};
use crate::types::user::{UserRegisterRequest, ChangePasswordRequest, UserProfile as TypesUserProfile, Gender};
use chrono::NaiveDate;

/// Authentication service managing user sessions and tokens
#[derive(Clone)]
pub struct AuthService {
    http_client: ApiClient,
    storage_service: StorageService,
}

impl AuthService {
    /// Create new authentication service
    pub fn new(http_client: ApiClient, storage_service: StorageService) -> Self {
        Self {
            http_client,
            storage_service,
        }
    }

    /// Check if user is currently authenticated
    pub fn is_authenticated(&self) -> bool {
        self.storage_service.get_token().is_some()
    }

    /// Get current user profile from storage
    pub fn get_current_user(&self) -> Option<UserProfile> {
        self.storage_service.get_user_profile()
    }

    /// Login user with email and password
    pub async fn login(&self, email: String, password: String) -> Result<UserProfile, AuthError> {
        self.validate_login_input(&email, &password)?;

        // Direct API call without wrapper
        let request = LoginRequest {
            email: email.trim().to_lowercase(),
            password: password.trim().to_string(),
        };

        let token_response: TypesTokenResponse = self.http_client
            .post(ApiEndpoints::AUTH_LOGIN, &request)
            .await
            .map_err(AuthError::from)?;

        // Convert and store token
        let dto_token = TokenResponse {
            token: token_response.token,
            expires_in: Some(token_response.exp as u64),
        };
        
        self.storage_service.store_token(&dto_token)
            .map_err(AuthError::from)?;

        // Get user profile
        let profile_response: TypesUserProfile = self.http_client
            .get(ApiEndpoints::USER_PROFILE)
            .await
            .map_err(AuthError::from)?;

        // Convert and store profile
        let user_profile = UserProfile {
            id: profile_response.id.to_string(),
            email: profile_response.email,
            full_name: profile_response.username,
            created_at: Some(profile_response.created_at.to_rfc3339()),
            updated_at: Some(profile_response.updated_at.to_rfc3339()),
        };

        self.storage_service.store_user_profile(&user_profile)
            .map_err(AuthError::from)?;

        Ok(user_profile)
    }

    /// Logout current user
    pub async fn logout(&self) -> Result<(), AuthError> {
        if self.storage_service.get_token().is_some() {
            // Direct API call
            let _: serde_json::Value = self.http_client
                .post(ApiEndpoints::AUTH_LOGOUT, &())
                .await
                .unwrap_or_else(|_| {
                    leptos::logging::warn!("API logout failed, proceeding with local cleanup");
                    serde_json::Value::Null
                });
        }

        self.storage_service.clear_session()
            .map_err(AuthError::from)?;

        Ok(())
    }

    /// Refresh authentication token
    pub async fn refresh_token(&self) -> Result<(), AuthError> {
        let _current_token = self.storage_service.get_token()
            .ok_or(AuthError::NotAuthenticated)?;

        let token_response: TypesTokenResponse = self.http_client
            .post(ApiEndpoints::AUTH_REFRESH, &())
            .await
            .map_err(AuthError::from)?;

        let dto_token = TokenResponse {
            token: token_response.token,
            expires_in: Some(token_response.exp as u64),
        };

        self.storage_service.store_token(&dto_token)
            .map_err(AuthError::from)?;

        Ok(())
    }

    /// Verify current token validity
    pub async fn verify_token(&self) -> Result<bool, AuthError> {
        let _token = self.storage_service.get_token()
            .ok_or(AuthError::NotAuthenticated)?;

        match self.http_client.get::<serde_json::Value>(ApiEndpoints::AUTH_VERIFY).await {
            Ok(_) => Ok(true),
            Err(HttpError::Unauthorized) => {
                self.storage_service.clear_session()
                    .map_err(AuthError::from)?;
                Ok(false)
            }
            Err(e) => Err(AuthError::from(e)),
        }
    }

    /// Register new user
    pub async fn register(&self, email: String, password: String, full_name: String) -> Result<UserProfile, AuthError> {
        self.validate_registration_input(&email, &password, &full_name)?;

        let request = UserRegisterRequest {
            email: email.trim().to_lowercase(),
            password: password.trim().to_string(),
            username: full_name.clone(),
            first_name: full_name.split_whitespace().next().unwrap_or("").to_string(),
            last_name: full_name.split_whitespace().skip(1).collect::<Vec<_>>().join(" "),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "".to_string(),
            gender: serde_json::to_value(Gender::Other).unwrap(),  // Converto enum in JSON
        };

        let profile_response: TypesUserProfile = self.http_client
            .post(ApiEndpoints::USER_REGISTER, &request)
            .await
            .map_err(AuthError::from)?;

        let user_profile = UserProfile {
            id: profile_response.id.to_string(),
            email: profile_response.email,
            full_name: profile_response.username,
            created_at: Some(profile_response.created_at.to_rfc3339()),
            updated_at: Some(profile_response.updated_at.to_rfc3339()),
        };

        self.storage_service.store_user_profile(&user_profile)
            .map_err(AuthError::from)?;

        Ok(user_profile)
    }

    /// Change user password
    pub async fn change_password(&self, current_password: String, new_password: String) -> Result<(), AuthError> {
        self.validate_password_change(&current_password, &new_password)?;

        let request = ChangePasswordRequest {
            current_password: current_password.trim().to_string(),
            new_password: new_password.trim().to_string(),
        };

        let _: serde_json::Value = self.http_client
            .post(ApiEndpoints::USER_CHANGE_PASSWORD, &request)
            .await
            .map_err(AuthError::from)?;

        Ok(())
    }

    /// Validate login input parameters
    fn validate_login_input(&self, email: &str, password: &str) -> Result<(), AuthError> {
        if email.trim().is_empty() {
            return Err(AuthError::InvalidInput("Email is required".to_string()));
        }

        if password.trim().is_empty() {
            return Err(AuthError::InvalidInput("Password is required".to_string()));
        }

        if !email.contains('@') {
            return Err(AuthError::InvalidInput("Invalid email format".to_string()));
        }

        Ok(())
    }

    /// Validate registration input parameters
    fn validate_registration_input(&self, email: &str, password: &str, full_name: &str) -> Result<(), AuthError> {
        self.validate_login_input(email, password)?;

        if full_name.trim().is_empty() {
            return Err(AuthError::InvalidInput("Full name is required".to_string()));
        }

        if password.len() < AuthConstants::MIN_PASSWORD_LENGTH {
            return Err(AuthError::InvalidInput(format!("Password must be at least {} characters", AuthConstants::MIN_PASSWORD_LENGTH)));
        }

        Ok(())
    }

    /// Validate password change parameters
    fn validate_password_change(&self, current_password: &str, new_password: &str) -> Result<(), AuthError> {
        if current_password.trim().is_empty() {
            return Err(AuthError::InvalidInput("Current password is required".to_string()));
        }

        if new_password.trim().is_empty() {
            return Err(AuthError::InvalidInput("New password is required".to_string()));
        }

        if new_password.len() < AuthConstants::MIN_PASSWORD_LENGTH {
            return Err(AuthError::InvalidInput(format!("New password must be at least {} characters", AuthConstants::MIN_PASSWORD_LENGTH)));
        }

        if current_password == new_password {
            return Err(AuthError::InvalidInput("New password must be different from current password".to_string()));
        }

        Ok(())
    }
}

impl Default for AuthService {
    fn default() -> Self {
        let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
        let storage_service = StorageService::new();
        Self::new(http_client, storage_service)
    }
}
