// Test factory for generating consistent test data

use std::sync::atomic::{AtomicU32, Ordering};
use chrono::NaiveDate;
use ruggine_client_ui::types::{
    user::{UserRegisterRequest, UserProfile, UserStatus, UserType, Gender, CurrentAction},
    auth::{LoginRequest, TokenResponse},
    group::{GroupChatCreateRequest, GroupChat},
    membership::GroupMembership,
    invitation::{MemberRole, MembershipStatus},
};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(1);

pub struct TestFactory;

impl TestFactory {
    /// Generate unique user information for tests
    pub fn get_unique_user_info(prefix: &str) -> (String, String, String) {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        let unique_suffix = format!("{}_{}", counter, timestamp);
        let email = format!("{}{}@test.com", prefix, unique_suffix);
        let username = format!("{}_{}", prefix, unique_suffix);
        let full_name = format!("{}{}", prefix, unique_suffix);
        
        (email, username, full_name)
    }

    /// Create unique UserRegisterRequest for testing
    pub fn unique_user_register_request(prefix: &str) -> UserRegisterRequest {
        let (email, username, _) = Self::get_unique_user_info(prefix);
        
        UserRegisterRequest {
            email,
            password: "testpass123".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            username,
            birthday: "1990-01-01".to_string(), // String format as required by server
            address: "123 Test Street".to_string(),
            gender: Gender::Male,
        }
    }

    /// Create unique LoginRequest for testing
    pub fn unique_login_request(prefix: &str) -> LoginRequest {
        let (email, _, _) = Self::get_unique_user_info(prefix);
        
        LoginRequest {
            email,
            password: "testpass123".to_string(),
        }
    }

    /// Create mock UserProfile for testing
    pub fn mock_user_profile(prefix: &str) -> UserProfile {
        let (email, username, _) = Self::get_unique_user_info(prefix);
        
        UserProfile {
            id: 1,
            email,
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            username,
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Test Street".to_string(),
            gender: Gender::Male,
            user_type: UserType::EndUser,  // Changed from User to EndUser
            user_status: UserStatus::Active,
            current_action: CurrentAction::Waiting,  // Changed from Online to Waiting
            is_online: true,  // Add missing field
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: Some(chrono::Utc::now()),
        }
    }

    /// Create mock TokenResponse for testing
    pub fn mock_token_response() -> TokenResponse {
        let now = chrono::Utc::now().timestamp();
        
        TokenResponse {
            token: "mock_jwt_token_12345".to_string(),
            iat: now,
            exp: now + 3600, // 1 hour from now
        }
    }

    /// Create expired TokenResponse for testing
    pub fn mock_expired_token_response() -> TokenResponse {
        let now = chrono::Utc::now().timestamp();
        
        TokenResponse {
            token: "expired_jwt_token_12345".to_string(),
            iat: now - 7200, // 2 hours ago
            exp: now - 3600, // 1 hour ago (expired)
        }
    }

    /// Standard test email
    pub fn test_email(prefix: &str) -> String {
        format!("{}@test.com", prefix)
    }

    /// Standard test username
    pub fn test_username(prefix: &str) -> String {
        format!("test_{}", prefix)
    }

    /// Create valid GroupChatCreateRequest for testing
    pub fn valid_group_create_request(prefix: &str) -> GroupChatCreateRequest {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        GroupChatCreateRequest {
            name: format!("Test Group {}{}", prefix, counter),
            description: format!("Test group description for {}{}", prefix, counter),
        }
    }

    /// Create minimal valid GroupChatCreateRequest
    pub fn minimal_group_create_request(prefix: &str) -> GroupChatCreateRequest {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        GroupChatCreateRequest {
            name: format!("Group{}", counter),
            description: "".to_string(), // Empty description (valid)
        }
    }

    /// Create GroupChatCreateRequest with specific parameters
    pub fn custom_group_create_request(name: &str, description: &str) -> GroupChatCreateRequest {
        GroupChatCreateRequest {
            name: name.to_string(),
            description: description.to_string(),
        }
    }

    /// Create mock GroupChat for testing
    pub fn mock_group_chat(prefix: &str) -> GroupChat {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let now = chrono::Utc::now();
        
        GroupChat {
            id: counter as i32,
            name: format!("Mock Group {}{}", prefix, counter),
            description: format!("Mock group description for {}{}", prefix, counter),
            created_by: 1,
            created_at: now,
            updated_at: now,
            member_count: Some(1), // Add missing field
            is_active: true, // Add missing field
        }
    }

    /// Create mock GroupMembership for testing
    pub fn mock_group_membership(prefix: &str) -> GroupMembership {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let now = chrono::Utc::now();
        
        GroupMembership {
            id: counter as i32,
            user_id: 1,
            group_chat_id: counter as i32,
            role: MemberRole::Member, // Use proper enum
            joined_at: now,
            left_at: None, // Add missing field
            membership_status: MembershipStatus::Active, // Add missing field
            invitation_id: counter as i32, // Add missing field
            group_name: Some(format!("Mock Group {}", counter)), // Add missing field
            user_name: Some(format!("Test User {}", counter)), // Add missing field
        }
    }

    /// Create multiple mock GroupMemberships
    pub fn mock_multiple_group_memberships(count: usize, prefix: &str) -> Vec<GroupMembership> {
        (0..count)
            .map(|i| {
                let mut membership = Self::mock_group_membership(&format!("{}_{}", prefix, i));
                membership.group_chat_id = (i + 1) as i32; // Different group IDs
                membership
            })
            .collect()
    }
}
