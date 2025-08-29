use crate::api::client::ApiClient;
use crate::utils::storage::StorageService;
use crate::error::AuthError;
use crate::config::{endpoints::ApiEndpoints, constants::AppConstants};
use crate::types::group::{GroupChatCreateRequest, ApiSuccessResponseGroupChatReadDto, GroupChat};
use crate::types::membership::{ApiSuccessResponseVecGroupMembershipReadDto, GroupMembership};
use serde::{Deserialize, Serialize};

/// Group chat management service
#[derive(Clone)]
pub struct GroupChatService {
    http_client: ApiClient,
    storage_service: StorageService,
}

impl GroupChatService {
    /// Create new group chat service
    pub fn new(http_client: ApiClient, storage_service: StorageService) -> Self {
        Self {
            http_client,
            storage_service,
        }
    }

    /// Create a new group chat
    pub async fn create_group(&self, request: GroupChatCreateRequest) -> Result<GroupChat, AuthError> {
        // Validate request
        self.validate_group_create_request(&request)?;

        // Send API request
        let response: ApiSuccessResponseGroupChatReadDto = self.http_client
            .post(ApiEndpoints::GROUP_CREATE, &request)
            .await
            .map_err(AuthError::from)?;

        // Convert to client-side GroupChat
        let group_chat = GroupChat::from(response);

        Ok(group_chat)
    }


    /// Get group by ID
    pub async fn get_group_by_id(&self, group_id: &str) -> Result<GroupChat, AuthError> {
        let response: ApiSuccessResponseGroupChatReadDto = self.http_client
            .get(&ApiEndpoints::group_by_id(group_id))
            .await
            .map_err(AuthError::from)?;

        Ok(GroupChat::from(response))
    }

    /// Get group memberships for the current user
    pub async fn get_user_groups(&self) -> Result<Vec<GroupMembership>, AuthError> {
        let response: ApiSuccessResponseVecGroupMembershipReadDto = self.http_client
            .get(ApiEndpoints::USER_GROUPS)
            .await
            .map_err(AuthError::from)?;

        Ok(Vec::from(response))
    }

    /// Update group information
    pub async fn update_group(&self, group_id: &str, update_request: GroupUpdateRequest) -> Result<GroupChat, AuthError> {
        let response: ApiSuccessResponseGroupChatReadDto = self.http_client
            .patch(&ApiEndpoints::group_by_id(group_id), &update_request)
            .await
            .map_err(AuthError::from)?;

        Ok(GroupChat::from(response))
    }

    /// Delete group
    pub async fn delete_group(&self, group_id: &str) -> Result<(), AuthError> {
        self.http_client
            .delete::<()>(&ApiEndpoints::group_by_id(group_id))
            .await
            .map_err(AuthError::from)?;

        Ok(())
    }

    /// Validate group creation request
    pub fn validate_group_create_request(&self, request: &GroupChatCreateRequest) -> Result<(), AuthError> {
        if request.name.trim().is_empty() {
            return Err(AuthError::ValidationError("Group name cannot be empty".to_string()));
        }

        if request.name.len() > 100 {
            return Err(AuthError::ValidationError("Group name cannot exceed 100 characters".to_string()));
        }

        if request.description.len() > 500 {
            return Err(AuthError::ValidationError("Group description cannot exceed 500 characters".to_string()));
        }

        Ok(())
    }
}

/// Group update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupUpdateRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl Default for GroupChatService {
    fn default() -> Self {
        let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
        let storage_service = StorageService::new();
        Self::new(http_client, storage_service)
    }
}
