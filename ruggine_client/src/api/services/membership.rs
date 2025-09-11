use crate::api::client::ApiClient;
use crate::utils::storage::StorageService;
use crate::error::AuthError;
use crate::config::{endpoints::ApiEndpoints, constants::AppConstants};
use crate::types::membership::{
    ApiSuccessResponseVecGroupMembershipReadDto,
    ApiSuccessResponseGroupMembershipReadDto,
    GroupMembership,
    ApiSuccessResponseVecUserId,
};
use serde::Serialize;
use once_cell::sync::Lazy;
use std::sync::RwLock;
use crate::utils::timers::now_ms;

// Simple in-memory cache: map group_id -> (timestamp_ms, data)
static GROUP_MEMBERSHIP_CACHE: Lazy<RwLock<std::collections::HashMap<i32, (u64, Vec<GroupMembership>)>>> = Lazy::new(|| RwLock::new(std::collections::HashMap::new()));
const CACHE_TTL_SECS: u64 = 5; // small TTL to avoid stale data but prevent rapid repeated calls

/// Group membership management service
#[derive(Clone)]
pub struct GroupMembershipService {
    http_client: ApiClient,
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
        // Try cache first
        if let Ok(id_val) = group_chat_id.parse::<i32>() {
            if let Ok(cache) = GROUP_MEMBERSHIP_CACHE.read() {
                if let Some((ts_ms, data)) = cache.get(&id_val) {
                    let elapsed = now_ms().saturating_sub(*ts_ms);
                    if elapsed < (CACHE_TTL_SECS * 1000) {
                        return Ok(data.clone());
                    }
                }
            }
        }

        let response: ApiSuccessResponseVecGroupMembershipReadDto = self.http_client
            .get(&ApiEndpoints::group_membership_by_group_chat(group_chat_id))
            .await
            .map_err(AuthError::from)?;
        let vec = Vec::from(response);

        // Store into cache
        if let Ok(id_val) = group_chat_id.parse::<i32>() {
            if let Ok(mut cache) = GROUP_MEMBERSHIP_CACHE.write() {
                cache.insert(id_val, (now_ms(), vec.clone()));
            }
        }

        Ok(vec)
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
    pub fn new(http_client: ApiClient, _storage_service: StorageService) -> Self {
        Self {
            http_client,
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

    /// Get connected users currently online (returns vector of user ids)
    /// Returns empty vector if user is not online or request fails
    pub async fn find_connected_users_and_online(&self) -> Result<Vec<i32>, AuthError> {
        match self.http_client
            .get::<ApiSuccessResponseVecUserId>(ApiEndpoints::group_membership_connected_users_online())
            .await {
            Ok(response) => Ok(response.data),
            Err(http_error) => {
                // Handle 403 Forbidden (user not online) gracefully
                match &http_error {
                    crate::api::http_error::HttpError::Http { status, .. } if *status == 403 => {
                        log::warn!("User is not online, returning empty online users list");
                        Ok(vec![]) // Return empty list instead of error
                    },
                    _ => Err(AuthError::from(http_error))
                }
            }
        }
    }
}

impl Default for GroupMembershipService {
    fn default() -> Self {
        let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
        let storage_service = StorageService::new();
        Self::new(http_client, storage_service)
    }
}
