use crate::api::client::ApiClient;
use crate::utils::storage::StorageService;
use crate::error::AuthError;
use crate::config::{endpoints::ApiEndpoints, constants::AppConstants};
use crate::types::membership::{
    ApiSuccessResponseVecGroupMembershipReadDto,
    ApiSuccessResponseGroupMembershipReadDto,
    GroupMembership,
    GroupMembershipReadDto,
};
use serde::Serialize;

/// Group membership management service
#[derive(Clone)]
pub struct GroupMembershipService {
    http_client: ApiClient,
    storage_service: StorageService,
}

impl GroupMembershipService {
    /// Get group membership by id
    pub async fn get_by_id(&self, membership_id: &str) -> Result<GroupMembership, AuthError> {
        let response: ApiSuccessResponseGroupMembershipReadDto = self.http_client
            .get(&format!("{}/{}", ApiEndpoints::GROUP_MEMBERSHIP_BY_ID, membership_id))
            .await
            .map_err(AuthError::from)?;
        Ok(GroupMembership::from(response))
    }

    /// Get group memberships by group_chat_id
    pub async fn get_by_group_chat_id(&self, group_chat_id: &str) -> Result<Vec<GroupMembership>, AuthError> {
        let response: ApiSuccessResponseVecGroupMembershipReadDto = self.http_client
            .get(&ApiEndpoints::group_membership_by_group_chat(group_chat_id))
            .await
            .map_err(AuthError::from)?;
        Ok(Vec::from(response))
    }

    /// Leave a group membership
    pub async fn leave_group(&self, membership_id: i32) -> Result<GroupMembership, AuthError> {
        #[derive(Serialize)]
        struct LeaveGroupMembershipDto {
            id: i32,
        }
        let body = LeaveGroupMembershipDto { id: membership_id };
        let response: ApiSuccessResponseGroupMembershipReadDto = self.http_client
            .patch(ApiEndpoints::GROUP_MEMBERSHIP_LEAVE, &body)
            .await
            .map_err(AuthError::from)?;
        Ok(GroupMembership::from(response))
    }
    /// Create new group membership service
    pub fn new(http_client: ApiClient, storage_service: StorageService) -> Self {
        Self {
            http_client,
            storage_service,
        }
    }

    /// Get user's group memberships
    pub async fn get_user_groups(&self) -> Result<Vec<GroupMembership>, AuthError> {
        let response: ApiSuccessResponseVecGroupMembershipReadDto = self.http_client
            .get(ApiEndpoints::GROUP_MEMBERSHIP_BY_USER)
            .await
            .map_err(AuthError::from)?;

        Ok(Vec::from(response))
    }

    // Qui puoi aggiungere altri metodi membership-specifici (leave, join, ecc.)
}

impl Default for GroupMembershipService {
    fn default() -> Self {
        let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
        let storage_service = StorageService::new();
        Self::new(http_client, storage_service)
    }
}
