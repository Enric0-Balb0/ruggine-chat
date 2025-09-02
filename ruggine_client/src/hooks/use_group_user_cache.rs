use leptos::*;
use crate::types::user::UserProfile;
use crate::api::services::GroupMembershipService;
use crate::api::services::UserService;
use crate::utils::storage::StorageService;
use std::collections::HashMap;
use crate::hooks::fetch_missing_users::fetch_missing_users;

/// Hook that provides a reactive cache of user profiles for a group.
pub fn use_group_user_cache(group_id: i32) -> RwSignal<HashMap<i32, UserProfile>> {
    let user_cache = create_rw_signal(HashMap::<i32, UserProfile>::new());
    let storage_service = StorageService::new();
    let http_client = crate::api::ApiClient::new(crate::config::constants::AppConstants::DEFAULT_SERVER_URL);
    if let Some(token_response) = storage_service.get_token() {
        http_client.set_auth_token(Some(token_response.token));
    }
    let membership_service = GroupMembershipService::new(http_client.clone(), storage_service.clone());
    let user_service = UserService::new(http_client.clone(), storage_service.clone());

    // Fetch group members on mount (batch fetch missing users into the cache)
    create_effect(move |_| {
        // Use untracked read so updates to `user_cache` inside the task don't retrigger this effect
        let user_cache_snapshot = user_cache.get_untracked();
        let user_cache = user_cache.clone();
        let membership_service = membership_service.clone();
        let user_service = user_service.clone();
        spawn_local(async move {
            match membership_service.get_by_group_chat_id(&group_id.to_string()).await {
                Ok(memberships) => {
                    // Collect ids that are not yet present in cache
                    let mut missing_ids = Vec::new();
                    for m in memberships {
                        let user_id = m.user_id;
                        if !user_cache_snapshot.contains_key(&user_id) {
                            missing_ids.push(user_id);
                        }
                    }

                    if !missing_ids.is_empty() {
                        // Use the batched fetch helper which deduplicates and guards in-flight requests
                        fetch_missing_users(missing_ids, user_cache.clone(), user_service.clone());
                    }
                }
                Err(_e) => {}
            }
        });
    });

    user_cache
}
