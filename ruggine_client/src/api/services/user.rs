use crate::api::client::ApiClient;
use crate::utils::storage::StorageService;
use crate::error::AuthError;
use crate::config::{endpoints::ApiEndpoints, constants::AppConstants};
use crate::types::user::{ApiSuccessResponseUserReadDto, UserReadDto};
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
    fn from(profile: UserReadDto) -> Self {
        Self {
            id: profile.id.to_string(),
            email: profile.email,
            username: profile.username,
            first_name: profile.first_name,
            last_name: profile.last_name,
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

    /// Get user profile by ID
    pub async fn get_user_by_id(&self, user_id: &str) -> Result<UserSearchResult, AuthError> {
        let profile_response: ApiSuccessResponseUserReadDto = self.http_client
            .get(&ApiEndpoints::user_by_id(user_id))
            .await
            .map_err(AuthError::from)?;

        Ok(UserSearchResult::from(profile_response.data))
    }

    /// Update current user profile
    pub async fn update_profile(&self, profile_update: UserProfileUpdate) -> Result<UserProfile, AuthError> {
        let response: ApiSuccessResponseUserReadDto = self.http_client
            .patch(ApiEndpoints::USER_UPDATE_PROFILE, &profile_update)
            .await
            .map_err(AuthError::from)?;

        // Convert using the automatic conversion from ApiSuccessResponseUserReadDto
        let user_profile = UserProfile::from(response);

        self.storage_service.store_user_profile(&user_profile)
            .map_err(AuthError::from)?;

        Ok(user_profile)
    }

    /// Get cached user profile
    pub fn get_cached_profile(&self) -> Option<UserProfile> {
        self.storage_service.get_user_profile()
    }

}

/// User profile update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileUpdate {
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub address: Option<String>,
}

impl Default for UserService {
    fn default() -> Self {
        let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
        let storage_service = StorageService::new();
        Self::new(http_client, storage_service)
    }
}
