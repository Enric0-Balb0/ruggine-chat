use crate::api::client::ApiClient;
use crate::utils::storage::StorageService;
use crate::error::AuthError;
use crate::config::{endpoints::ApiEndpoints, constants::AppConstants};
use crate::types::user::{UserReadDto, ApiSuccessResponseUserReadDto};
use crate::types::UserProfile;
use serde::{Deserialize, Serialize};

/// User search and management service
#[derive(Clone)]
pub struct UserService {
    http_client: ApiClient,
    storage_service: StorageService,
}

/// User search result for invitation purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchResult {
    pub id: String,
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
}

impl From<UserReadDto> for UserSearchResult {
    fn from(user_data: UserReadDto) -> Self {
        Self {
            id: user_data.id.to_string(),
            email: user_data.email,
            username: user_data.username,
            first_name: user_data.first_name,
            last_name: user_data.last_name,
        }
    }
}

impl UserService {
    /// Create new user service
    pub fn new(http_client: ApiClient, storage_service: StorageService) -> Self {
        Self {
            http_client,
            storage_service,
        }
    }

    /// Get user profile by username (new OpenAPI)
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<UserSearchResult>, AuthError> {
        // Prova prima il formato diretto (UserReadDto)
        match self.http_client
            .get::<UserReadDto>(&ApiEndpoints::user_by_username(username))
            .await {
            Ok(user_data) => {
                web_sys::console::log_1(&format!("Successfully found user (direct format): {:?}", user_data).into());
                Ok(Some(UserSearchResult {
                    id: user_data.id.to_string(),
                    username: user_data.username,
                    first_name: user_data.first_name,
                    last_name: user_data.last_name,
                    email: user_data.email,
                }))
            },
            Err(http_error) => {
                web_sys::console::log_1(&format!("Direct format failed: {:?}", http_error).into());
                
                // Se il formato diretto fallisce, prova il formato wrapped
                match self.http_client
                    .get::<crate::types::user::ApiSuccessResponseUserReadDto>(&ApiEndpoints::user_by_username(username))
                    .await {
                    Ok(wrapped_response) => {
                        web_sys::console::log_1(&format!("Wrapped format response: {:?}", wrapped_response).into());
                        let user_data = wrapped_response.data;
                        web_sys::console::log_1(&format!("Successfully found user (wrapped format): {:?}", user_data).into());
                        Ok(Some(UserSearchResult {
                            id: user_data.id.to_string(),
                            username: user_data.username,
                            first_name: user_data.first_name,
                            last_name: user_data.last_name,
                            email: user_data.email,
                        }))
                    },
                    Err(wrapped_error) => {
                        web_sys::console::log_1(&format!("Both formats failed. Direct error: {:?}, Wrapped error: {:?}", http_error, wrapped_error).into());
                        // Gestisci 404 come utente non trovato
                        match &http_error {
                            crate::api::http_error::HttpError::Http { status, .. } if *status == 404 => {
                                web_sys::console::log_1(&"User not found (404)".into());
                                Ok(None)
                            },
                            _ => Err(AuthError::from(http_error))
                        }
                    }
                }
            }
        }
    }

    /// Get user profile by ID
    pub async fn get_user_by_id(&self, user_id: &str) -> Result<Option<UserSearchResult>, AuthError> {
        // Prova prima il formato diretto (UserReadDto)
        match self.http_client
            .get::<UserReadDto>(&ApiEndpoints::user_by_id(user_id))
            .await {
            Ok(user_data) => {
                web_sys::console::log_1(&format!("Direct format success for user {}", user_id).into());
                Ok(Some(UserSearchResult {
                    id: user_data.id.to_string(),
                    email: user_data.email,
                    username: user_data.username,
                    first_name: user_data.first_name,
                    last_name: user_data.last_name,
                }))
            },
            Err(http_error) => {
                web_sys::console::log_1(&format!("Direct format failed for user {}, trying wrapped format", user_id).into());
                // Se fallisce, prova il formato wrappato
                match self.http_client
                    .get::<crate::types::user::ApiSuccessResponseUserReadDto>(&ApiEndpoints::user_by_id(user_id))
                    .await {
                    Ok(response) => {
                        let user_data = response.data;
                        web_sys::console::log_1(&format!("Wrapped format success for user {}", user_id).into());
                        Ok(Some(UserSearchResult::from(user_data)))
                    },
                    Err(wrapped_error) => {
                        web_sys::console::log_1(&format!("Both formats failed for user {}. Direct error: {:?}, Wrapped error: {:?}", user_id, http_error, wrapped_error).into());
                        // Gestisci 404 come utente non trovato
                        match &http_error {
                            crate::api::http_error::HttpError::Http { status, .. } if *status == 404 => {
                                web_sys::console::log_1(&format!("User {} not found (404)", user_id).into());
                                Ok(None)
                            },
                            _ => Err(AuthError::from(http_error))
                        }
                    }
                }
            }
        }
    }

    /// Update current user profile
    pub async fn update_profile(&self, profile_update: UserProfileUpdate) -> Result<UserProfile, AuthError> {
        let response: ApiSuccessResponseUserReadDto = self.http_client
            .patch(ApiEndpoints::USER_UPDATE_PROFILE, &profile_update)
            .await
            .map_err(AuthError::from)?;

        // Get user data directly (no longer Option)
        let user_data = response.data;

        // Convert UserReadDto to UserProfile manually
        let user_profile = UserProfile {
            id: user_data.id,
            email: user_data.email,
            first_name: user_data.first_name,
            last_name: user_data.last_name,
            username: user_data.username,
            birthday: user_data.birthday.parse().unwrap_or_default(),
            address: user_data.address,
            gender: user_data.gender,
            user_type: user_data.user_type,
            user_status: user_data.user_status,
            current_action: crate::types::user::CurrentAction::Waiting,
            is_online: user_data.is_online,
            created_at: user_data.created_at.parse().unwrap_or_default(),
            updated_at: user_data.updated_at.parse().unwrap_or_default(),
            last_login: None,
        };

        self.storage_service.store_user_profile(&user_profile)
            .map_err(AuthError::from)?;

        Ok(user_profile)
    }

    /// Get cached user profile
    pub fn get_cached_profile(&self) -> Option<UserProfile> {
        self.storage_service.get_user_profile()
    }

    /// Get current user profile from server
    pub async fn get_current_profile(&self) -> Result<UserProfile, AuthError> {
        // Get current token for authorization
        let token = self.storage_service.get_token()
            .ok_or_else(|| AuthError::NotAuthenticated)?;
        
        // Set auth token for the request
        self.http_client.set_auth_token(Some(token.token));
        
        let profile_response: ApiSuccessResponseUserReadDto = self.http_client
            .get(ApiEndpoints::USER_PROFILE)
            .await
            .map_err(AuthError::from)?;
        
        // Use the existing conversion from ApiSuccessResponseUserReadDto to UserProfile
        let user_profile = UserProfile::from(profile_response);
        
        Ok(user_profile)
    }
}

/// User profile update request (corrisponde a ProfileUpdateDto OpenAPI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileUpdate {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address: Option<String>,
    pub birthday: Option<String>,
    pub gender: Option<String>,
}

impl Default for UserService {
    fn default() -> Self {
        let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
        let storage_service = StorageService::new();
        Self::new(http_client, storage_service)
    }
}