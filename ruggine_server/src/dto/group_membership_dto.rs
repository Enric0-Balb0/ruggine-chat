use crate::entity::group_membership::{GroupMembership, MemberRole, MembershipStatus, NewGroupMembership, UpdateGroupMembership};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, PartialEq, Eq)]
#[schema(example = json!({
    "user_id": 1,
    "group_chat_id": 1
}))]
pub struct CreateGroupMembershipDto {
    #[schema(example = 1)]
    pub user_id: i32,
    #[schema(example = 1)]
    pub group_chat_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, PartialEq, Eq)]
#[schema(example = json!({
    "user_id": 1,
    "group_chat_id": 1
}))]
pub struct CreateAdminGroupMembershipDto {
    #[schema(example = 1)]
    pub user_id: i32,
    #[schema(example = 1)]
    pub group_chat_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, PartialEq, Eq)]
#[schema(example = json!({
    "group_chat_id": 1
}))]
pub struct LeaveGroupMembershipDto {
    #[schema(example = "1")]
    pub group_chat_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[schema(example = json!({
    "id": 1,
    "user_id": 1,
    "group_chat_id": 1,
    "role": "member",
    "joined_at": "2024-01-01T12:00:00Z",
    "left_at": null,
    "membership_status": "active"
}))]
pub struct GroupMembershipReadDto {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = 1)]
    pub user_id: i32,
    #[schema(example = 1)]
    pub group_chat_id: i32,
    #[schema(example = "member")]
    pub role: MemberRole,
    #[schema(example = "2024-01-01T12:00:00Z")]
    pub joined_at: DateTime<Utc>,
    #[schema(example = "null")]
    pub left_at: Option<DateTime<Utc>>,
    #[schema(example = "active")]
    pub membership_status: MembershipStatus,
}

impl From<GroupMembership> for GroupMembershipReadDto {
    fn from(membership: GroupMembership) -> Self {
        Self {
            id: membership.id,
            user_id: membership.user_id,
            group_chat_id: membership.group_chat_id,
            role: membership.role,
            joined_at: membership.joined_at,
            left_at: membership.left_at,
            membership_status: membership.membership_status,
        }
    }
}

impl CreateGroupMembershipDto {
    pub fn to_new_group_membership(&self) -> NewGroupMembership {
        NewGroupMembership {
            user_id: self.user_id,
            group_chat_id: self.group_chat_id,
            role: Default::default(), // Default role is Member
        }
    }
}

impl CreateAdminGroupMembershipDto {
    pub fn to_new_group_membership(&self) -> NewGroupMembership {
        NewGroupMembership {
            user_id: self.user_id,
            group_chat_id: self.group_chat_id,
            role: MemberRole::Admin,
        }
    }
}

impl LeaveGroupMembershipDto {
    pub fn to_update_group_membership(&self, user_id: i32) -> UpdateGroupMembership {
        UpdateGroupMembership {
            user_id: user_id,
            group_chat_id: self.group_chat_id,
            membership_status: Some(MembershipStatus::Left),
            left_at: Some(Utc::now()), // Set left_at to now
            role: None, // Role is not updated when leaving
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_group_membership_dto_to_new_group_membership() {
        let dto = CreateGroupMembershipDto {
            user_id: 1,
            group_chat_id: 1,
        };
        
        let new_membership = dto.to_new_group_membership();
        assert_eq!(new_membership.user_id, 1);
        assert_eq!(new_membership.group_chat_id, 1);
        assert_eq!(new_membership.role, MemberRole::Member); // Default role
    }

    #[test]
    fn test_create_admin_group_membership_dto_to_new_group_membership() {
        let dto = CreateAdminGroupMembershipDto {
            user_id: 1,
            group_chat_id: 1,
        };
        
        let new_membership = dto.to_new_group_membership();
        assert_eq!(new_membership.user_id, 1);
        assert_eq!(new_membership.group_chat_id, 1);
        assert_eq!(new_membership.role, MemberRole::Admin);
    }
    
    #[test]
    fn test_leave_group_membership_dto() {
        let dto = LeaveGroupMembershipDto {
            group_chat_id: 1,
        };
        
        assert_eq!(dto.group_chat_id, 1);
    }

    #[test]
    fn test_leave_group_membership_dto_to_update_group_membership() {
        let dto = LeaveGroupMembershipDto {
            group_chat_id: 1,
        };
        
        let update = dto.to_update_group_membership(2);
        assert_eq!(update.user_id, 2);
        assert_eq!(update.group_chat_id, 1);
        assert_eq!(update.membership_status, Some(MembershipStatus::Left));
        assert!(update.left_at.is_some()); // left_at should be set to now
        assert_eq!(update.role, None); // Role is not updated when leaving
    }

    #[test]
    fn test_group_membership_read_dto_from() {
        let membership = GroupMembership {
            id: 1,
            user_id: 1,
            group_chat_id: 1,
            role: MemberRole::Admin,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
        };
        
        let read_dto = GroupMembershipReadDto::from(membership.clone());
        assert_eq!(read_dto.id, membership.id);
        assert_eq!(read_dto.user_id, membership.user_id);
        assert_eq!(read_dto.group_chat_id, membership.group_chat_id);
        assert_eq!(read_dto.role, membership.role);
        assert_eq!(read_dto.joined_at, membership.joined_at);
        assert_eq!(read_dto.left_at, membership.left_at);
        assert_eq!(read_dto.membership_status, membership.membership_status);

    }

    #[test]
    fn test_group_membership_read_dto_with_left_at() {
        let left_time = Utc::now();
        let membership = GroupMembership {
            id: 2,
            user_id: 2,
            group_chat_id: 2,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: Some(left_time),
            membership_status: MembershipStatus::Left,
        };
        
        let read_dto = GroupMembershipReadDto::from(membership);
        assert_eq!(read_dto.id, 2);
        assert_eq!(read_dto.user_id, 2);
        assert_eq!(read_dto.group_chat_id, 2);
        assert_eq!(read_dto.role, MemberRole::Member);
        assert_eq!(read_dto.left_at, Some(left_time));
        assert_eq!(read_dto.membership_status, MembershipStatus::Left);
    }

    #[test]
    fn test_dto_serialization() {
        let create_dto = CreateGroupMembershipDto {
            user_id: 1,
            group_chat_id: 1,
        };
        
        let json = serde_json::to_string(&create_dto).unwrap();
        let deserialized: CreateGroupMembershipDto = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.user_id, create_dto.user_id);
        assert_eq!(deserialized.group_chat_id, create_dto.group_chat_id);
    }

    #[test]
    fn test_admin_dto_serialization() {
        let admin_dto = CreateAdminGroupMembershipDto {
            user_id: 2,
            group_chat_id: 3,
        };
        
        let json = serde_json::to_string(&admin_dto).unwrap();
        let deserialized: CreateAdminGroupMembershipDto = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.user_id, admin_dto.user_id);
        assert_eq!(deserialized.group_chat_id, admin_dto.group_chat_id);
    }

    #[test]
    fn test_leave_group_membership_dto_serialization() {
        let dto = LeaveGroupMembershipDto {
            group_chat_id: 1,
        };

        let json = serde_json::to_string(&dto).unwrap();
        let deserialized: LeaveGroupMembershipDto = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.group_chat_id, dto.group_chat_id);
    }
}
