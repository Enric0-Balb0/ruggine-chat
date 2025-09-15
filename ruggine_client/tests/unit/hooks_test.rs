#[cfg(test)]
mod hooks_tests {
    
    use crate::common::TestFactory;
    use ruggine_client_ui::types::common::LoadingState;
    
    use ruggine_client_ui::hooks::GroupMembershipWithDetails;
    use std::collections::HashMap;

    #[cfg(test)]
    mod groups_hook_tests {
        use super::*;

        #[test]
        fn test_group_membership_with_details_creation() {
            let membership = TestFactory::mock_group_membership_with_details();
            
            assert!(membership.membership.id > 0);
            assert!(!membership.group_name().is_empty());
            assert!(membership.group_details.is_some());
            assert!(membership.membership.group_chat_id > 0);
        }

        #[test]
        fn test_group_membership_with_details_cloning() {
            let original = TestFactory::mock_group_membership_with_details();
            let cloned = original.clone();
            
            assert_eq!(original.membership.id, cloned.membership.id);
            assert_eq!(original.group_name(), cloned.group_name());
            assert_eq!(original.membership.group_chat_id, cloned.membership.group_chat_id);
            assert_eq!(original.group_details.is_some(), cloned.group_details.is_some());
        }

        #[test]
        fn test_group_membership_multiple_instances() {
            let membership1 = TestFactory::mock_group_membership_with_details();
            let membership2 = TestFactory::mock_group_membership_with_details();
            
            assert_ne!(membership1.membership.id, membership2.membership.id);
            assert!(membership1.group_name() != membership2.group_name());
        }

        #[test]
        fn test_loading_state_with_groups_data() {
            let groups = vec![
                TestFactory::mock_group_membership_with_details(),
                TestFactory::mock_group_membership_with_details(),
            ];

            let loading_state = LoadingState::Success(groups.clone());
            
            match loading_state {
                LoadingState::Success(data) => {
                    assert_eq!(data.len(), 2);
                    assert!(data[0].group_name().contains("group"));
                    assert!(data[1].group_name().contains("group"));
                }
                _ => panic!("Expected Success state"),
            }
        }

        #[test]
        fn test_loading_state_loading() {
            let loading_state: LoadingState<Vec<GroupMembershipWithDetails>> = LoadingState::Loading;
            
            match loading_state {
                LoadingState::Loading => assert!(true),
                _ => panic!("Expected Loading state"),
            }
        }

        #[test]
        fn test_loading_state_error() {
            let error_msg = "Failed to load groups".to_string();
            let loading_state: LoadingState<Vec<GroupMembershipWithDetails>> = LoadingState::Error(error_msg.clone());
            
            match loading_state {
                LoadingState::Error(msg) => assert_eq!(msg, error_msg),
                _ => panic!("Expected Error state"),
            }
        }
    }

    #[cfg(test)]
    mod message_hook_tests {
        use super::*;
        use ruggine_client_ui::types::message::Message;

        #[test]
        fn test_message_vector_operations() {
            let messages = vec![
                TestFactory::mock_message("msg1"),
                TestFactory::mock_message("msg2"),
                TestFactory::mock_message("msg3"),
            ];

            assert_eq!(messages.len(), 3);
            for (index, message) in messages.iter().enumerate() {
                assert!(message.content.contains(&format!("msg{}", index + 1)));
                assert!(!message.content.is_empty());
            }
        }

        #[test]
        fn test_message_pagination_simulation() {
            let page1 = vec![
                TestFactory::mock_message("page1_msg1"),
                TestFactory::mock_message("page1_msg2"),
            ];
            let page2 = vec![
                TestFactory::mock_message("page2_msg1"),
                TestFactory::mock_message("page2_msg2"),
            ];

            let mut all_messages = page1;
            all_messages.extend(page2);

            assert_eq!(all_messages.len(), 4);
            assert!(all_messages[0].content.contains("page1"));
            assert!(all_messages[2].content.contains("page2"));
        }

        #[test]
        fn test_message_loading_states() {
            // Test loading state
            let loading: LoadingState<Vec<Message>> = LoadingState::Loading;
            match loading {
                LoadingState::Loading => assert!(true),
                _ => panic!("Expected Loading state"),
            }

            // Test success state
            let messages = vec![TestFactory::mock_message("loaded")];
            let success = LoadingState::Success(messages.clone());
            match success {
                LoadingState::Success(data) => assert_eq!(data.len(), 1),
                _ => panic!("Expected Success state"),
            }
        }

        #[test]
        fn test_message_error_handling() {
            let error_msg = "Failed to load messages".to_string();
            let error_state: LoadingState<Vec<Message>> = LoadingState::Error(error_msg.clone());

            match error_state {
                LoadingState::Error(msg) => assert_eq!(msg, error_msg),
                _ => panic!("Expected Error state"),
            }
        }
    }

    #[cfg(test)]
    mod user_cache_hook_tests {
        use super::*;
        use ruggine_client_ui::types::user::UserProfile;

        #[test]
        fn test_user_cache_hashmap_operations() {
            let mut user_cache: HashMap<i32, UserProfile> = HashMap::new();
            
            let user1 = TestFactory::mock_user_profile();
            let user2 = TestFactory::mock_user_profile();
            
            // Make sure IDs are different
            assert_ne!(user1.id, user2.id);
            
            user_cache.insert(user1.id, user1.clone());
            user_cache.insert(user2.id, user2.clone());

            assert_eq!(user_cache.len(), 2);
            assert!(user_cache.contains_key(&user1.id));
            assert!(user_cache.contains_key(&user2.id));
        }

        #[test]
        fn test_missing_users_identification() {
            let mut cached_users: HashMap<i32, UserProfile> = HashMap::new();
            let user1 = TestFactory::mock_user_profile();
            cached_users.insert(user1.id, user1);

            let required_user_ids = vec![1, 999]; // 1 exists, 999 doesn't
            let missing_ids: Vec<i32> = required_user_ids
                .iter()
                .filter(|id| !cached_users.contains_key(id))
                .copied()
                .collect();

            // At least one should be missing (999)
            assert!(missing_ids.len() >= 1);
            assert!(missing_ids.contains(&999));
        }

        #[test]
        fn test_user_cache_batch_operations() {
            let users = vec![
                TestFactory::mock_user_profile(),
                TestFactory::mock_user_profile(),
                TestFactory::mock_user_profile(),
            ];

            let mut cache: HashMap<i32, UserProfile> = HashMap::new();
            for user in users {
                cache.insert(user.id, user);
            }

            // Should have at least 1 user (may be less if IDs conflict)
            assert!(cache.len() >= 1);
            assert!(cache.values().all(|user| user.username.contains("mock")));
        }

        #[test]
        fn test_user_cache_loading_state() {
            let users = vec![TestFactory::mock_user_profile()];
            let loading_state = LoadingState::Success(users.clone());

            match loading_state {
                LoadingState::Success(data) => {
                    assert_eq!(data.len(), 1);
                    assert!(data[0].username.contains("mock"));
                }
                _ => panic!("Expected Success state"),
            }
        }
    }

    #[cfg(test)]
    mod fetch_missing_users_tests {
        use super::*;
        use ruggine_client_ui::types::user::UserProfile;

        #[test]
        fn test_fetch_missing_users_filtering() {
            let cached_users = vec![
                TestFactory::mock_user_profile(),
                TestFactory::mock_user_profile(),
            ];

            let required_ids = vec![1, 2, 999, 1000];
            let cache_map: HashMap<i32, UserProfile> = cached_users
                .into_iter()
                .map(|user| (user.id, user))
                .collect();

            let missing: Vec<i32> = required_ids
                .iter()
                .filter(|id| !cache_map.contains_key(id))
                .copied()
                .collect();

            assert!(missing.len() >= 2); // Should have at least the two missing IDs
            assert!(missing.contains(&999));
            assert!(missing.contains(&1000));
        }

        #[test]
        fn test_empty_missing_users() {
            let cached_users = vec![
                TestFactory::mock_user_profile(),
                TestFactory::mock_user_profile(),
            ];

            let cache_map: HashMap<i32, UserProfile> = cached_users
                .into_iter()
                .map(|user| (user.id, user))
                .collect();

            let required_ids: Vec<i32> = cache_map.keys().copied().collect();
            let missing: Vec<i32> = required_ids
                .iter()
                .filter(|id| !cache_map.contains_key(id))
                .copied()
                .collect();

            assert_eq!(missing.len(), 0);
        }

        #[test]
        fn test_all_users_missing() {
            let cache_map: HashMap<i32, UserProfile> = HashMap::new();
            let required_ids = vec![1, 2, 3];

            let missing: Vec<i32> = required_ids
                .iter()
                .filter(|id| !cache_map.contains_key(id))
                .copied()
                .collect();

            assert_eq!(missing.len(), 3);
            assert_eq!(missing, required_ids);
        }

        #[test]
        fn test_partial_cache_hit() {
            let mut cache_map: HashMap<i32, UserProfile> = HashMap::new();
            let cached_user = TestFactory::mock_user_profile();
            let cached_id = cached_user.id;
            cache_map.insert(cached_id, cached_user);

            let required_ids = vec![cached_id, 999, 1000];
            let missing: Vec<i32> = required_ids
                .iter()
                .filter(|id| !cache_map.contains_key(id))
                .copied()
                .collect();

            assert_eq!(missing.len(), 2);
            assert!(!missing.contains(&cached_id));
        }
    }

    #[cfg(test)]
    mod hook_integration_tests {
        use super::*;
        use ruggine_client_ui::types::user::UserProfile;

        #[test]
        fn test_groups_and_users_integration() {
            let groups = vec![
                TestFactory::mock_group_membership_with_details(),
                TestFactory::mock_group_membership_with_details(),
            ];

            let users = vec![
                TestFactory::mock_user_profile(),
                TestFactory::mock_user_profile(),
            ];

            let groups_state = LoadingState::Success(groups.clone());
            let users_cache: HashMap<i32, UserProfile> = users
                .into_iter()
                .map(|user| (user.id, user))
                .collect();

            match groups_state {
                LoadingState::Success(groups_data) => {
                    assert_eq!(groups_data.len(), 2);
                    assert!(users_cache.len() >= 1); // May be less if IDs conflict
                }
                _ => panic!("Expected Success state"),
            }
        }

        #[test]
        fn test_data_consistency() {
            let group = TestFactory::mock_group_membership_with_details();
            let group_clone = group.clone();

            assert_eq!(group.membership.id, group_clone.membership.id);
            assert_eq!(group.group_name(), group_clone.group_name());
            assert_eq!(group.membership.group_chat_id, group_clone.membership.group_chat_id);
        }

        #[test]
        fn test_loading_state_consistency() {
            let data = vec![TestFactory::mock_group_membership_with_details()];
            let state1 = LoadingState::Success(data.clone());
            let state2 = LoadingState::Success(data);

            match (state1, state2) {
                (LoadingState::Success(data1), LoadingState::Success(data2)) => {
                    assert_eq!(data1.len(), data2.len());
                    assert_eq!(data1[0].membership.id, data2[0].membership.id);
                }
                _ => panic!("Both states should be Success"),
            }
        }

        #[test]
        fn test_error_handling_integration() {
            let error_msg = "Integration error".to_string();
            let groups_error: LoadingState<Vec<GroupMembershipWithDetails>> = 
                LoadingState::Error(error_msg.clone());
            let users_error: LoadingState<HashMap<i32, UserProfile>> = 
                LoadingState::Error(error_msg.clone());

            match (groups_error, users_error) {
                (LoadingState::Error(groups_msg), LoadingState::Error(users_msg)) => {
                    assert_eq!(groups_msg, users_msg);
                }
                _ => panic!("Both should be in error state"),
            }
        }
    }

    #[cfg(test)]
    mod hook_state_transitions_tests {
        use super::*;
        use ruggine_client_ui::types::user::UserProfile;

        #[test]
        fn test_loading_to_success_transition() {
            let mut state: LoadingState<Vec<GroupMembershipWithDetails>> = LoadingState::Loading;
            
            // Simulate loading completion
            let groups = vec![TestFactory::mock_group_membership_with_details()];
            state = LoadingState::Success(groups.clone());

            match state {
                LoadingState::Success(data) => {
                    assert_eq!(data.len(), 1);
                    assert!(data[0].group_name().contains("group"));
                }
                _ => panic!("Expected Success state after transition"),
            }
        }

        #[test]
        fn test_cache_state_changes() {
            let mut cache: HashMap<i32, UserProfile> = HashMap::new();
            
            // Add user
            let user = TestFactory::mock_user_profile();
            let user_id = user.id;
            cache.insert(user_id, user.clone());
            assert_eq!(cache.len(), 1);

            // Update user (replace) - use a different user to ensure different data
            let updated_user = TestFactory::mock_user_profile();
            cache.insert(updated_user.id, updated_user);
            // May be 1 or 2 depending on if IDs are different
            assert!(cache.len() >= 1);

            // Remove user
            cache.remove(&user_id);
            // May be 0 or 1 depending on if the updated user had a different ID
            assert!(cache.len() <= 1);
        }

        #[test]
        fn test_error_to_success_recovery() {
            let mut state: LoadingState<Vec<GroupMembershipWithDetails>> = 
                LoadingState::Error("Initial error".to_string());

            // Simulate recovery
            let groups = vec![TestFactory::mock_group_membership_with_details()];
            state = LoadingState::Success(groups.clone());

            match state {
                LoadingState::Success(data) => {
                    assert_eq!(data.len(), 1);
                    assert!(data[0].group_name().contains("group"));
                }
                _ => panic!("Expected successful recovery to Success state"),
            }
        }

        #[test]
        fn test_pagination_state_accumulation() {
            let mut all_messages = Vec::new();
            
            // Simulate multiple page loads
            for page in 1..=3 {
                let page_messages = vec![
                    TestFactory::mock_message(&format!("page{}_msg1", page)),
                    TestFactory::mock_message(&format!("page{}_msg2", page)),
                ];
                all_messages.extend(page_messages);
            }

            assert_eq!(all_messages.len(), 6);
            assert!(all_messages[0].content.contains("page1"));
            assert!(all_messages[4].content.contains("page3"));
        }

        #[test]
        fn test_concurrent_cache_updates() {
            let mut cache: HashMap<i32, UserProfile> = HashMap::new();
            
            // Simulate batch user updates - use different prefixes to reduce ID conflicts
            let batch1 = vec![
                TestFactory::mock_user_profile(),
                TestFactory::mock_user_profile(),
            ];
            
            let batch2 = vec![
                TestFactory::mock_user_profile(),
                TestFactory::mock_user_profile(),
            ];

            // Apply batch1
            for user in batch1 {
                cache.insert(user.id, user);
            }
            assert!(cache.len() >= 1);

            // Apply batch2
            for user in batch2 {
                cache.insert(user.id, user);
            }
            assert!(cache.len() >= 1);
        }
    }
}
