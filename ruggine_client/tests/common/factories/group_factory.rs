// Group and membership-related test data factory

use super::base_factory::BaseFactory;
use chrono::DateTime;
use ruggine_client_ui::types::{
    group::{GroupChatCreateRequest, GroupChat},
    membership::{GroupMembership, CurrentAction},
    invitation::{MemberRole, MembershipStatus},
};

pub struct GroupFactory;

impl GroupFactory {
    /// Create minimal group create request
    pub fn minimal_group_create_request(prefix: &str) -> GroupChatCreateRequest {
        let group_info = BaseFactory::get_unique_group_info(prefix);
        
        GroupChatCreateRequest {
            name: group_info.0,
            description: group_info.1,
        }
    }
    
    /// Create basic group create request
    pub fn basic_group_create_request() -> GroupChatCreateRequest {
        GroupChatCreateRequest {
            name: "Test Group".to_string(),
            description: "A test group for testing purposes".to_string(),
        }
    }
    
    /// Create mock group chat
    pub fn mock_group_chat() -> GroupChat {
        let unique_id = BaseFactory::get_unique_id();
        let group_info = BaseFactory::get_unique_group_info("mock");
        
        GroupChat {
            id: unique_id as i32,
            name: group_info.0,
            description: group_info.1,
            created_by: 1,
            created_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            updated_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            member_count: Some(5),
            is_active: true,
        }
    }
    
    /// Create group with specific details
    pub fn group_with_details(id: i32, name: &str, created_by: i32) -> GroupChat {
        GroupChat {
            id,
            name: name.to_string(),
            description: format!("Description for {}", name),
            created_by,
            created_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            updated_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            member_count: Some(3),
            is_active: true,
        }
    }
    
    /// Create mock group membership
    pub fn mock_group_membership(prefix: &str) -> GroupMembership {
        let unique_id = BaseFactory::get_unique_id();
        
        GroupMembership {
            id: unique_id as i32,
            user_id: 1,
            group_chat_id: unique_id as i32,
            role: MemberRole::Member,
            joined_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: 1,
            current_action: CurrentAction::Waiting,
            group_name: Some(format!("{}group{}", prefix, unique_id)),
            user_name: Some(format!("{}user{}", prefix, unique_id)),
        }
    }
    
    /// Create admin group membership
    pub fn admin_group_membership(user_id: i32, group_chat_id: i32) -> GroupMembership {
        GroupMembership {
            id: BaseFactory::get_unique_id() as i32,
            user_id,
            group_chat_id,
            role: MemberRole::Admin,
            joined_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: BaseFactory::get_unique_id() as i32,
            current_action: CurrentAction::Writing,
            group_name: None,
            user_name: None,
        }
    }
    
    /// Create member group membership
    pub fn member_group_membership(user_id: i32, group_chat_id: i32) -> GroupMembership {
        GroupMembership {
            id: BaseFactory::get_unique_id() as i32,
            user_id,
            group_chat_id,
            role: MemberRole::Member,
            joined_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: BaseFactory::get_unique_id() as i32,
            current_action: CurrentAction::Waiting,
            group_name: None,
            user_name: None,
        }
    }
    
    /// Create membership with details
    pub fn membership_with_details(user_id: i32, group_chat_id: i32, role: MemberRole) -> GroupMembership {
        let unique_id = BaseFactory::get_unique_id() as i32;
        
        GroupMembership {
            id: unique_id,
            user_id,
            group_chat_id,
            role,
            joined_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: unique_id,
            current_action: CurrentAction::Waiting,
            group_name: None,
            user_name: None,
        }
    }
    
    /// Create custom group create request with specific name and description
    pub fn custom_group_create_request(name: &str, description: &str) -> GroupChatCreateRequest {
        GroupChatCreateRequest {
            name: name.to_string(),
            description: description.to_string(),
        }
    }
}
