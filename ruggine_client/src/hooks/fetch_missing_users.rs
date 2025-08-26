use leptos::*;
use crate::types::user::UserProfile;
use crate::api::services::UserService;
use std::collections::HashMap;

/// Fetch user profiles for a list of sender_ids not present in the cache.
pub fn fetch_missing_users(
    sender_ids: Vec<i32>,
    user_cache: RwSignal<HashMap<i32, UserProfile>>,
    user_service: UserService,
) {
    for sender_id in sender_ids {
        if !user_cache.get().contains_key(&sender_id) {
            let user_cache = user_cache.clone();
            let user_service = user_service.clone();
            leptos::spawn_local(async move {
                if let Ok(profile) = user_service.get_user_by_id(&sender_id.to_string()).await {
                    let user_profile = UserProfile {
                        id: sender_id,
                        email: profile.email,
                        first_name: profile.first_name,
                        last_name: profile.last_name,
                        username: profile.username,
                        birthday: chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                        address: String::new(),
                        gender: crate::types::user::Gender::Other,
                        user_type: crate::types::user::UserType::EndUser,
                        user_status: crate::types::user::UserStatus::Active,
                        is_online: false,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        last_login: None,
                    };
                    user_cache.update(|cache| {
                        cache.insert(sender_id, user_profile);
                    });
                }
            });
        }
    }
}
