use leptos::*;
use crate::types::user::UserProfile;
use crate::api::services::GroupMembershipService;
use crate::api::services::UserService;
use crate::utils::storage::StorageService;
use std::collections::HashMap;

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

    // Fetch group members on mount
    create_effect(move |_| {
        let user_cache_snapshot = user_cache.get(); // tracked here!
        let user_cache = user_cache.clone();
        let membership_service = membership_service.clone();
        let user_service = user_service.clone();
        spawn_local(async move {
            match membership_service.get_by_group_chat_id(&group_id.to_string()).await {
                Ok(memberships) => {
                    for m in memberships {
                        let user_id = m.user_id;
                        // Avoid duplicates
                        if !user_cache_snapshot.contains_key(&user_id) {
                            if let Ok(profile) = user_service.get_user_by_id(&user_id.to_string()).await {
                                // Convert UserSearchResult -> UserProfile (partial, only basic fields)
                                let user_profile = UserProfile {
                                    id: user_id,
                                    email: profile.email,
                                    first_name: profile.first_name,
                                    last_name: profile.last_name,
                                    username: profile.username,
                                    birthday: chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                                    address: String::new(),
                                    gender: crate::types::user::Gender::Other,
                                    user_type: crate::types::user::UserType::EndUser,
                                    user_status: crate::types::user::UserStatus::Active,
                                    current_action: crate::types::membership::CurrentAction::Waiting,
                                    is_online: false,
                                    created_at: chrono::Utc::now(),
                                    updated_at: chrono::Utc::now(),
                                    last_login: None,
                                };
                                user_cache.update(|cache| {
                                    cache.insert(user_id, user_profile);
                                });
                            }
                        }
                    }
                }
                Err(_e) => {}
            }
        });
    });

    user_cache
}
