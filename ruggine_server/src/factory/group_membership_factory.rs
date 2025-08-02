use std::sync::{atomic::{AtomicU32, Ordering}};
use chrono::Utc;

use crate::{
    dto::group_membership_dto::{CreateAdminGroupMembershipDto, CreateGroupMembershipDto, GroupMembershipReadDto, LeaveGroupMembershipDto},
    entity::group_membership::{GroupMembership, MemberRole, MembershipStatus, NewGroupMembership, UpdateGroupMembership}
};

// Global counter for unique test data
static TEST_COUNTER: AtomicU32 = AtomicU32::new(1);

pub struct GroupMembershipFactory;

impl GroupMembershipFactory {

    pub fn fake_new_group_membership() -> NewGroupMembership {
        NewGroupMembership {
            user_id: 1,
            group_chat_id: 1,
            role: MemberRole::Member,
        }
    }

    pub fn fake_new_admin_group_membership() -> NewGroupMembership {
        NewGroupMembership {
            user_id: 1,
            group_chat_id: 1,
            role: MemberRole::Admin,
        }
    }

    pub fn fake_new_group_membership_with_ids(user_id: i32, group_chat_id: i32) -> NewGroupMembership {
        NewGroupMembership {
            user_id,
            group_chat_id,
            role: MemberRole::Member,
        }
    }

    pub fn fake_new_admin_group_membership_with_ids(user_id: i32, group_chat_id: i32) -> NewGroupMembership {
        NewGroupMembership {
            user_id,
            group_chat_id,
            role: MemberRole::Admin,
        }
    }

    pub fn fake_group_membership() -> GroupMembership {
        GroupMembership {
            id: 1,
            user_id: 1,
            group_chat_id: 1,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
        }
    }

    pub fn fake_left_group_membership() -> GroupMembership {
        let mut membership = Self::fake_group_membership();
        membership.left_at = Some(Utc::now());
        membership.membership_status = MembershipStatus::Left;
        membership
    }

    pub fn fake_admin_group_membership() -> GroupMembership {
        let mut membership = Self::fake_group_membership();
        membership.role = MemberRole::Admin;
        membership
    }

    pub fn fake_group_membership_from_ids(user_id: i32, group_chat_id: i32) -> GroupMembership {
        let counter: u32 = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        GroupMembership {
            id: counter as i32,
            user_id,
            group_chat_id,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
        }
    }

    pub fn fake_left_group_membership_from_ids(user_id: i32, group_chat_id: i32) -> GroupMembership {
        let mut membership = Self::fake_group_membership_from_ids(user_id, group_chat_id);
        membership.left_at = Some(Utc::now());
        membership.membership_status = MembershipStatus::Left;
        membership
    }

    pub fn fake_admin_group_membership_from_ids(user_id: i32, group_chat_id: i32) -> GroupMembership {
        let mut membership = Self::fake_group_membership_from_ids(user_id, group_chat_id);
        membership.role = MemberRole::Admin;
        membership
    }

    pub fn fake_group_membership_with_role(role: MemberRole) -> GroupMembership {
        let mut membership = Self::fake_group_membership();
        membership.role = role;
        membership
    }

    pub fn fake_update_group_membership() -> UpdateGroupMembership {
        UpdateGroupMembership {
            user_id: 1,
            group_chat_id: 1,
            role: Some(MemberRole::Admin),
            left_at: None,
            membership_status: None,
        }
    }

    pub fn fake_leave_group_membership() -> UpdateGroupMembership {
        UpdateGroupMembership {
            user_id: 1,
            group_chat_id: 1,
            role: None,
            left_at: Some(Utc::now()),
            membership_status: Some(MembershipStatus::Left),
        }
    }

    pub fn fake_create_group_membership_dto() -> CreateGroupMembershipDto {
        CreateGroupMembershipDto {
            user_id: 1,
            group_chat_id: 1,
        }
    }

    pub fn fake_create_admin_group_membership_dto() -> CreateAdminGroupMembershipDto {
        CreateAdminGroupMembershipDto {
            user_id: 1,
            group_chat_id: 1,
        }
    }

    pub fn fake_leave_group_membership_dto() -> LeaveGroupMembershipDto {
        LeaveGroupMembershipDto {
            group_chat_id: 1,
        }
    }

    pub fn fake_group_membership_read_dto() -> GroupMembershipReadDto {
        GroupMembershipReadDto {
            id: 1,
            user_id: 1,
            group_chat_id: 1,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
        }
    }

    pub fn fake_leave_group_membership_read_dto() -> GroupMembershipReadDto {
        GroupMembershipReadDto {
            id: 1,
            user_id: 1,
            group_chat_id: 1,
            role: MemberRole::Member,
            joined_at: Utc::now() - chrono::Duration::days(1),
            left_at: Some(Utc::now()),
            membership_status: MembershipStatus::Left,
        }
    }

    pub fn fake_group_membership_read_dto_from_membership(membership: &GroupMembership) -> GroupMembershipReadDto {
        GroupMembershipReadDto {
            id: membership.id,
            user_id: membership.user_id,
            group_chat_id: membership.group_chat_id,
            role: membership.role.clone(),
            joined_at: membership.joined_at,
            left_at: membership.left_at,
            membership_status: membership.membership_status.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_new_group_membership() {
        let membership = GroupMembershipFactory::fake_new_group_membership();
        assert_eq!(membership.user_id, 1);
        assert_eq!(membership.group_chat_id, 1);
        assert_eq!(membership.role, MemberRole::Member);
    }

    #[test]
    fn test_fake_new_admin_group_membership() {
        let membership = GroupMembershipFactory::fake_new_admin_group_membership();
        assert_eq!(membership.user_id, 1);
        assert_eq!(membership.group_chat_id, 1);
        assert_eq!(membership.role, MemberRole::Admin);
    }

    #[test]
    fn test_fake_group_membership() {
        let membership = GroupMembershipFactory::fake_group_membership();
        assert_eq!(membership.id, 1);
        assert_eq!(membership.user_id, 1);
        assert_eq!(membership.group_chat_id, 1);
        assert_eq!(membership.role, MemberRole::Member);
        assert!(membership.left_at.is_none());
    }

    #[test]
    fn test_fake_admin_group_membership() {
        let membership = GroupMembershipFactory::fake_admin_group_membership();
        assert_eq!(membership.role, MemberRole::Admin);
    }

    #[test]
    fn test_fake_left_group_membership() {
        let membership = GroupMembershipFactory::fake_left_group_membership();
        assert!(membership.left_at.is_some());
    }
}
