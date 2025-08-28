use leptos::*;
use crate::api::services::{GroupChatService, GroupMembershipService};
use crate::types::membership::GroupMembership;
use crate::types::group::GroupChat;
use crate::types::common::LoadingState;
use crate::utils::storage::StorageService;
use crate::api::client::ApiClient;
use crate::config::constants::AppConstants;

/// Enhanced GroupMembership with group details
#[derive(Debug, Clone, PartialEq)]
pub struct GroupMembershipWithDetails {
    pub membership: GroupMembership,
    pub group_details: Option<GroupChat>,
}

impl GroupMembershipWithDetails {
    pub fn group_name(&self) -> String {
        self.group_details
            .as_ref()
            .map(|g| g.name.clone())
            .unwrap_or_else(|| format!("Group {}", self.membership.group_chat_id))
    }
}

/// Hook for managing user's groups
#[derive(Clone)]
pub struct UseGroups {
    pub groups: ReadSignal<LoadingState<Vec<GroupMembershipWithDetails>>>,
    pub active_group_id: ReadSignal<Option<i32>>,
    pub refresh_groups: Action<(), Result<Vec<GroupMembershipWithDetails>, String>>,
    pub set_active_group: WriteSignal<Option<i32>>,
}

pub fn use_groups() -> UseGroups {
    // Create group service
    let storage_service = StorageService::new();
    let http_client = ApiClient::new(AppConstants::DEFAULT_SERVER_URL);
    
    // Ensure the API client has the current token from storage
    if let Some(token_response) = storage_service.get_token() {
        http_client.set_auth_token(Some(token_response.token));
    }
    
    let group_service = GroupChatService::new(http_client.clone(), storage_service.clone());
    let membership_service = GroupMembershipService::new(http_client, storage_service);

    // State for groups
    let (groups, set_groups) = create_signal(LoadingState::Idle);
    
    // State for active group
    let (active_group_id, set_active_group) = create_signal(None::<i32>);

    // Action to refresh groups
    let refresh_groups = create_action(move |_: &()| {
    let group_service = group_service.clone();
    let membership_service = membership_service.clone();
        async move {
            
            
            let memberships = match membership_service.get_user_groups().await {
                Ok(memberships) => memberships,
                Err(e) => {
                    let error_msg = format!("Failed to fetch groups: {:?}", e);
                    logging::error!("{}", error_msg);
                    set_groups.set(LoadingState::Error(error_msg.clone()));
                    return Err(error_msg);
                }
            };

            

            let mut groups_with_details = Vec::new();
            for membership in memberships {
                let group_id = membership.group_chat_id.to_string();
                // Fetch group details
                let mut group_details = match group_service.get_group_by_id(&group_id).await {
                    Ok(details) => {
                        
                        Some(details)
                    },
                    Err(e) => {
                        logging::warn!("Failed to fetch details for group {}: {:?}", group_id, e);
                        None // Continue without details
                    }
                };

                // Fetch group membership count and set member_count if possible
                if let Some(ref mut group) = group_details {
                    match membership_service.get_by_group_chat_id(&group_id).await {
                        Ok(members) => {
                            group.member_count = Some(members.len() as i32);
                        },
                        Err(e) => {
                            logging::warn!("Failed to fetch membership count for group {}: {:?}", group_id, e);
                        }
                    }
                }

                groups_with_details.push(GroupMembershipWithDetails {
                    membership,
                    group_details,
                });
            }

            
            set_groups.set(LoadingState::Success(groups_with_details.clone()));
            Ok(groups_with_details)
        }
    });

    // Update groups state based on action
    create_effect(move |_| {
        match refresh_groups.value().get() {
            Some(Ok(_)) => {
                // Groups loaded successfully, state is already updated in the action
            },
            Some(Err(_)) => {
                // Error occurred, state is already updated in the action
            },
            None => {
                // Action not yet executed or in progress
                if refresh_groups.pending().get() {
                    set_groups.set(LoadingState::Loading);
                }
            }
        }
    });

    UseGroups {
        groups,
        active_group_id,
        refresh_groups,
        set_active_group,
    }
}

/// Derived signal to get groups as a simple Vec for rendering
pub fn use_groups_list(use_groups: &UseGroups) -> Memo<Vec<GroupMembershipWithDetails>> {
    let groups_signal = use_groups.groups;
    
    create_memo(move |_| {
        match groups_signal.get() {
            LoadingState::Success(groups) => groups,
            _ => vec![],
        }
    })
}

/// Check if groups are loading
pub fn use_groups_loading(use_groups: &UseGroups) -> Memo<bool> {
    let groups_signal = use_groups.groups;
    
    create_memo(move |_| {
        matches!(groups_signal.get(), LoadingState::Loading)
    })
}

/// Get groups error if any
pub fn use_groups_error(use_groups: &UseGroups) -> Memo<Option<String>> {
    let groups_signal = use_groups.groups;
    
    create_memo(move |_| {
        match groups_signal.get() {
            LoadingState::Error(error) => Some(error),
            _ => None,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_loading_state_transitions() {
        // Test that LoadingState variants work as expected
        let idle = LoadingState::<Vec<GroupMembershipWithDetails>>::Idle;
        let loading = LoadingState::<Vec<GroupMembershipWithDetails>>::Loading;
        let success: LoadingState<Vec<GroupMembershipWithDetails>> = LoadingState::Success(vec![]);
        let error: LoadingState<Vec<GroupMembershipWithDetails>> = LoadingState::Error("Test error".to_string());

        assert!(idle.data().is_none());
        assert!(loading.data().is_none());
        assert!(success.data().is_some());
        assert!(error.data().is_none());
        
        assert!(!idle.is_loading());
        assert!(loading.is_loading());
        assert!(!success.is_loading());
        assert!(!error.is_loading());
    }
}
