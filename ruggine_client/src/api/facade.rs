use crate::api::client::ApiClient;
use crate::api::services::{AuthService, UserService, GroupChatService};
use crate::utils::storage::StorageService;
use crate::types::UserProfile;
use crate::config::constants::AppConstants;

/// Main API client facade providing unified access to all services
#[derive(Clone)]
pub struct RuggineApiClient {
    pub auth_service: AuthService,
    pub user_service: UserService,
    pub group_service: GroupChatService,
    pub http_client: ApiClient,
    pub storage_service: StorageService,
}

impl RuggineApiClient {
    /// Create new API client instance
    pub fn new() -> Self {
        let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
        let storage_service = StorageService::new();
        let auth_service = AuthService::new(http_client.clone(), storage_service.clone());
        let user_service = UserService::new(http_client.clone(), storage_service.clone());
        let group_service = GroupChatService::new(http_client.clone(), storage_service.clone());

        Self {
            auth_service,
            user_service,
            group_service,
            http_client,
            storage_service,
        }
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth_service.is_authenticated()
    }

    /// Get current user profile
    pub fn get_current_user(&self) -> Option<UserProfile> {
        self.auth_service.get_current_user()
    }
}

impl Default for RuggineApiClient {
    fn default() -> Self {
        Self::new()
    }
}
