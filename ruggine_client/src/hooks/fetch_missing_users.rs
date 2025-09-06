use leptos::*;
use crate::types::user::UserProfile;
use crate::api::services::UserService;
use std::collections::{HashMap, HashSet};
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Track in-flight fetches to avoid issuing duplicate requests for the same user id
static IN_FLIGHT_FETCHES: Lazy<Mutex<HashSet<i32>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Fetch user profiles for a list of sender_ids not present in the cache.
/// This function deduplicates input ids and prevents concurrent duplicate requests
/// by using a static in-flight guard.
pub fn fetch_missing_users(
    sender_ids: Vec<i32>,
    user_cache: RwSignal<HashMap<i32, UserProfile>>,
    user_service: UserService,
) {
    // Deduplicate incoming ids
    let mut unique: Vec<i32> = sender_ids.into_iter().collect::<HashSet<_>>().into_iter().collect();
    if unique.is_empty() {
        return;
    }

    // Filter out ids already present in the cache
    // Non-reactive read: we just need a snapshot, no subscription
    unique.retain(|id| !user_cache.get_untracked().contains_key(id));
    if unique.is_empty() {
        return;
    }

    // Acquire lock and filter out ids that are already being fetched
    let mut to_fetch: Vec<i32> = Vec::new();
    {
        let mut guard = IN_FLIGHT_FETCHES.lock().unwrap();
        for id in unique.into_iter() {
            if !guard.contains(&id) {
                guard.insert(id);
                to_fetch.push(id);
            }
        }
    }

    if to_fetch.is_empty() {
        return;
    }

    // Spawn a single background task to fetch the remaining ids sequentially
    let user_cache = user_cache.clone();
    let user_service = user_service.clone();
    leptos::spawn_local(async move {
        for sender_id in to_fetch.iter() {
            let sid = *sender_id;
            if let Ok(profile) = user_service.get_user_by_id(&sid.to_string()).await {
                let user_profile = UserProfile {
                    id: sid,
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
                    cache.insert(sid, user_profile);
                });
            }
            // Remove from in-flight set so future calls can fetch if needed
            let mut guard = IN_FLIGHT_FETCHES.lock().unwrap();
            guard.remove(&sid);
        }
    });
}
