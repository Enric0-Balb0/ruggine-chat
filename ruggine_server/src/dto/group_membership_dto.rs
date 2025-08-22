use crate::entity::{group_membership::{GroupMembership, MemberRole, MembershipStatus, NewGroupMembership, UpdateGroupMembership}, invitation::Invitation};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;
use utoipa::ToSchema;
use crate::entity::group_membership::CurrentAction;
use crate::model::group_membership_model::GroupMembershipWithInvitationRow;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, PartialEq, Eq)]
#[schema(example = json!({
    "invitation_id": 1
}))]
pub struct GroupMembershipCreateDto {
    #[schema(example = 1)]
    #[validate(range(min = 1, message = "Invitation ID must be positive"))]
    pub invitation_id: i32,
    #[schema(example = "member")]
    pub role: MemberRole,

}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema, PartialEq, Eq)]
#[schema(example = json!({
    "id": 1
}))]
pub struct LeaveGroupMembershipDto {
    #[schema(example = "1")]
    #[validate(range(min = 1, message = "GroupMembership ID must be positive"))]
    pub id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[schema(example = json!({
    "id": 1,
    "user_id": 1,
    "group_chat_id": 1,
    "role": "member",
    "joined_at": "2024-01-01T12:00:00Z",
    "left_at": null,
    "membership_status": "active",
    "invitation_id": 1,
    "current_action": "waiting"
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
    #[schema(example = 1)]
    pub invitation_id: i32, // Foreign key to Invitation
    #[schema(example = "waiting")]
    pub current_action: CurrentAction,
}

impl From<GroupMembershipWithInvitationRow> for GroupMembershipReadDto {
    fn from(row: GroupMembershipWithInvitationRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            group_chat_id: row.group_chat_id,
            role: row.role,
            joined_at: row.joined_at,
            left_at: row.left_at,
            membership_status: row.membership_status,
            invitation_id: row.invitation_id,
            current_action: row.current_action,
        }
    }
}


impl GroupMembershipCreateDto {
    pub fn to_new_group_membership(&self) -> NewGroupMembership {
        NewGroupMembership {
            invitation_id: self.invitation_id,
            role: self.role.clone(),
        }
    }
}

impl LeaveGroupMembershipDto {
    pub fn to_update_group_membership(&self) -> UpdateGroupMembership {
        UpdateGroupMembership {
            membership_status: Some(MembershipStatus::Left),
            left_at: Some(Utc::now()), // Set left_at to now
            role: None, // Role is not updated when leaving
            id: self.id,
            current_action: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_create_group_membership_dto_to_new_group_membership() {
        let dto = GroupMembershipCreateDto {
            invitation_id: 1,
            role: MemberRole::Admin,
        };

        let new_membership = dto.to_new_group_membership();
        assert_eq!(new_membership.invitation_id, 1);
        assert_eq!(new_membership.role, MemberRole::Admin);
    }

    #[test]
    fn test_leave_group_membership_dto_to_update_group_membership() {
        let dto = LeaveGroupMembershipDto { id: 42 };

        let update = dto.to_update_group_membership();
        assert_eq!(update.id, 42);
        assert_eq!(update.membership_status, Some(MembershipStatus::Left));
        assert!(update.left_at.is_some()); // left_at should be set to now
        assert_eq!(update.role, None); // Role is not updated when leaving
    }

    #[test]
    fn test_group_membership_read_dto_from() {
        let row = GroupMembershipWithInvitationRow {
            id: 1,
            user_id: 2,
            group_chat_id: 99,
            role: MemberRole::Admin,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: 10,
            current_action: CurrentAction::Writing,
        };

        let read_dto = GroupMembershipReadDto::from(row.clone());

        assert_eq!(read_dto.id, row.id);
        assert_eq!(read_dto.user_id, row.user_id);
        assert_eq!(read_dto.group_chat_id, row.group_chat_id);
        assert_eq!(read_dto.role, row.role);
        assert_eq!(read_dto.joined_at, row.joined_at);
        assert_eq!(read_dto.left_at, row.left_at);
        assert_eq!(read_dto.membership_status, row.membership_status);
        assert_eq!(read_dto.invitation_id, row.invitation_id);
        assert_eq!(read_dto.current_action, row.current_action);
    }

    #[test]
    fn test_group_membership_read_dto_with_left_at() {
        let left_time = Utc::now();

        let row = GroupMembershipWithInvitationRow {
            id: 2,
            user_id: 6,
            group_chat_id: 88,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: Some(left_time),
            membership_status: MembershipStatus::Left,
            invitation_id: 77,
            current_action: CurrentAction::Writing,
        };

        let read_dto = GroupMembershipReadDto::from(row.clone());

        assert_eq!(read_dto.id, row.id);
        assert_eq!(read_dto.user_id, row.user_id);
        assert_eq!(read_dto.group_chat_id, row.group_chat_id);
        assert_eq!(read_dto.role, row.role);
        assert_eq!(read_dto.left_at, row.left_at);
        assert_eq!(read_dto.membership_status, row.membership_status);
        assert_eq!(read_dto.invitation_id, row.invitation_id);
        assert_eq!(read_dto.current_action, row.current_action);
    }

    #[test]
    fn test_dto_serialization() {
        let create_dto = GroupMembershipCreateDto {
            invitation_id: 1,
            role: MemberRole::Member,
        };

        let json = serde_json::to_string(&create_dto).unwrap();
        let deserialized: GroupMembershipCreateDto = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.invitation_id, create_dto.invitation_id);
        assert_eq!(deserialized.role, create_dto.role);
    }

    #[test]
    fn test_admin_dto_serialization() {
        let admin_dto = GroupMembershipCreateDto {
            invitation_id: 42,
            role: MemberRole::Admin,
        };

        let json = serde_json::to_string(&admin_dto).unwrap();
        let deserialized: GroupMembershipCreateDto = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.invitation_id, admin_dto.invitation_id);
        assert_eq!(deserialized.role, admin_dto.role);
    }

    #[test]
    fn test_leave_group_membership_dto_serialization() {
        let dto = LeaveGroupMembershipDto { id: 3 };

        let json = serde_json::to_string(&dto).unwrap();
        let deserialized: LeaveGroupMembershipDto = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, dto.id);
    }
}
