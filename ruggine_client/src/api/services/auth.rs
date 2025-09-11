use crate::api::{client::ApiClient, http_error::HttpError};
use crate::utils::storage::StorageService;
use crate::error::AuthError;
use crate::types::{UserProfile, TokenResponse};
use crate::config::{constants::{AuthConstants, AppConstants}, endpoints::ApiEndpoints};

// Direct type imports
use crate::types::auth::{LoginRequest, ApiSuccessResponseTokenReadDto};
use crate::types::user::{UserRegisterRequest, ChangePasswordRequest, ApiSuccessResponseUserReadDto, Gender};

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

    /// Check if user is currently authenticated with valid token
    pub fn is_authenticated(&self) -> bool {
        if let Some(token) = self.storage_service.get_token() {
            // Get current timestamp
            let now = chrono::Utc::now().timestamp();
            
            // Check if token exists and is not expired
            let is_valid = now < token.exp;
            
            log::debug!("AuthService::is_authenticated: token trovato, now={}, exp={}, iat={}, is_valid={}", 
                now, token.exp, token.iat, is_valid);
            
            // If token is expired, clean up storage
            if !is_valid {
                log::warn!("AuthService::is_authenticated: Token scaduto, pulizia storage");
                let _ = self.storage_service.remove_token();  // Rimuove da entrambi i storage
                return false;
            }
            
            // Additional check: ensure token is not issued in the future
            if now < token.iat {
                log::warn!("AuthService::is_authenticated: Token emesso nel futuro, pulizia storage");
                let _ = self.storage_service.remove_token();  // Rimuove da entrambi i storage
                return false;
            }
            
            is_valid
        } else {
            log::debug!("AuthService::is_authenticated: Nessun token trovato");
            false
        }
    }

    /// Clear user session (logout without API call)
    pub fn clear_session(&self) -> Result<(), AuthError> {
        // Clear ALL session data (both sessionStorage and localStorage)
        self.storage_service.clear_all();
        Ok(())
    }

    /// Check if token needs refresh (within threshold before expiry)
    pub fn needs_token_refresh(&self) -> bool {
        if let Some(token) = self.storage_service.get_token() {
            // Use the same adaptive logic as should_refresh_token
            crate::hooks::should_refresh_token(&token)
        } else {
            false
        }
    }

    /// Get current user profile from storage
    pub fn get_current_user(&self) -> Option<UserProfile> {
        self.storage_service.get_user_profile()
    }

    /// Get reference to storage service (for testing)
    pub fn get_storage_service(&self) -> &StorageService {
        &self.storage_service
    }

    /// Login user with email and password
    pub async fn login(&self, email: String, password: String) -> Result<UserProfile, AuthError> {
        self.validate_login_input(&email, &password)?;
        self.http_client.set_auth_token(None);

        let request = LoginRequest {
            email: email.trim().to_lowercase(),
            password: password.trim().to_string(),
        };

        let token_response: ApiSuccessResponseTokenReadDto = self.http_client
            .post(ApiEndpoints::AUTH_LOGIN, &request)
            .await
            .map_err(AuthError::from)?;

        // Convert token but don't store it yet - let the caller decide how to store it
        let dto_token = TokenResponse {
            token: token_response.data.token.clone(),
            iat: token_response.data.iat,
            exp: token_response.data.exp,
        };
        
        // Store the token in a temporary location for the caller to retrieve
        self.storage_service.store_token(&dto_token)
            .map_err(|e| AuthError::Storage(e))?;

        // Update the HTTP client with the new token for subsequent requests
        self.http_client.set_auth_token(Some(token_response.data.token));

        // Get user profile using the authenticated client
        let profile_response: ApiSuccessResponseUserReadDto = self.http_client
            .get(ApiEndpoints::USER_PROFILE)
            .await
            .map_err(AuthError::from)?;

        // Debug log the response data
        log::info!("AuthService::login: User data received - ID: {}, Email: {}", 
            profile_response.data.id, profile_response.data.email);

        // Convert using the automatic conversion from ApiSuccessResponseUserReadDto
        let user_profile = UserProfile::from(profile_response);

        log::info!("AuthService::login: Profilo ricevuto dal server - nome: '{}', cognome: '{}', email: '{}'",
            user_profile.first_name, user_profile.last_name, user_profile.email);

        // Non salvare il profilo qui - lascia che la landing page decida dove salvarlo
        // basandosi sulla modalità Remember Me
        log::info!("AuthService::login: Profilo non salvato - sarà gestito dalla landing page");

        Ok(user_profile)
    }

    /// Logout current user
    pub async fn logout(&self) -> Result<(), AuthError> {
        // Clear ALL session data (both sessionStorage and localStorage)
        self.storage_service.clear_all();
        Ok(())
    }

    /// Refresh authentication token
    pub async fn refresh_token(&self) -> Result<(), AuthError> {
    // The backend does not expose a token refresh endpoint.
    // Keep this method for API compatibility but return an explicit NotAuthenticated error
    // to indicate refresh is not supported client-side via HTTP call.
    Err(AuthError::NotAuthenticated)
    }

    /// Verify current token validity
    pub async fn verify_token(&self) -> Result<bool, AuthError> {
        let _token = self.storage_service.get_token()
            .ok_or(AuthError::NotAuthenticated)?;

        match self.http_client.get::<serde_json::Value>(ApiEndpoints::AUTH_VERIFY).await {
            Ok(_) => Ok(true),
            Err(HttpError::Unauthorized) => {
                // Clear ALL data on unauthorized (both sessionStorage and localStorage)
                self.storage_service.clear_all();
                Ok(false)
            }
            Err(e) => Err(AuthError::from(e)),
        }
    }

    /// Register new user with complete profile information
    pub async fn register(&self, 
        email: String, 
        password: String, 
        first_name: String,
        last_name: String,
        username: String,
        birthday: String,
        address: String,
        gender: String
    ) -> Result<UserProfile, AuthError> {
        self.validate_registration_input(&email, &password, &first_name)?;

        // Convert gender string to enum
        let gender_enum = match gender.as_str() {
            "male" => Gender::Male,
            "female" => Gender::Female,
            "other" => Gender::Other,
            _ => return Err(AuthError::InvalidInput("Genere non valido".to_string())),
        };

        let request = UserRegisterRequest {
            email: email.trim().to_lowercase(),
            password: password.trim().to_string(),
            first_name: first_name.trim().to_string(),
            last_name: last_name.trim().to_string(),
            username: username.trim().to_string(),
            birthday,
            address: address.trim().to_string(),
            gender: gender_enum,
        };

        let profile_response: ApiSuccessResponseUserReadDto = self.http_client
            .post(ApiEndpoints::USER_REGISTER, &request)
            .await
            .map_err(AuthError::from)?;

        // Convert using the automatic conversion from ApiSuccessResponseUserReadDto
        let user_profile = UserProfile::from(profile_response);

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
    fn validate_registration_input(&self, email: &str, password: &str, first_name: &str) -> Result<(), AuthError> {
        self.validate_login_input(email, password)?;

        if first_name.trim().is_empty() {
            return Err(AuthError::InvalidInput("Nome è obbligatorio".to_string()));
        }

        if password.len() < 6 {
            return Err(AuthError::InvalidInput("La password deve essere di almeno 6 caratteri".to_string()));
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
        
        let service = Self::new(http_client, storage_service);
        
        // If there's a stored token, load it into the HTTP client
        if let Some(stored_token) = service.storage_service.get_token() {
            service.http_client.set_auth_token(Some(stored_token.token));
        }
        
        service
    }
}
