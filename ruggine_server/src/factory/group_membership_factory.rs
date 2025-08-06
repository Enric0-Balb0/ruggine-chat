use std::sync::{atomic::{AtomicU32, Ordering}};
use chrono::Utc;

use crate::{
    dto::group_membership_dto::{CreateAdminGroupMembershipDto, GroupMembershipCreateDto, GroupMembershipReadDto, LeaveGroupMembershipDto},
    entity::group_membership::{GroupMembership, MemberRole, MembershipStatus, NewGroupMembership, UpdateGroupMembership}
};
use crate::entity::invitation::Invitation;
use crate::model::group_membership_model::GroupMembershipWithInvitationRow;

// Global counter for unique test data
static TEST_COUNTER: AtomicU32 = AtomicU32::new(1);

pub struct GroupMembershipFactory;

impl GroupMembershipFactory {

    pub fn fake_new_group_membership() -> NewGroupMembership {
        NewGroupMembership {
            invitation_id: 1,
            role: MemberRole::Member,
        }
    }

    pub fn fake_new_admin_group_membership() -> NewGroupMembership {
        NewGroupMembership {
            invitation_id: 1,
            role: MemberRole::Admin,
        }
    }

    pub fn fake_new_group_membership_with_id(invitation_id: i32) -> NewGroupMembership {
        NewGroupMembership {
            invitation_id,
            role: MemberRole::Member,
        }
    }

    pub fn fake_new_admin_group_membership_with_id(invitation_id: i32) -> NewGroupMembership {
        NewGroupMembership {
            invitation_id,
            role: MemberRole::Admin,
        }
    }

    pub fn fake_group_membership() -> GroupMembership {
        GroupMembership {
            id: 1,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: 1,
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

    pub fn fake_group_membership_from_id(invitation_id: i32) -> GroupMembership {
        let counter: u32 = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        GroupMembership {
            id: counter as i32,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id,
        }
    }

    pub fn fake_left_group_membership_from_ids(invitation_id: i32) -> GroupMembership {
        let mut membership = Self::fake_group_membership_from_id(invitation_id);
        membership.left_at = Some(Utc::now());
        membership.membership_status = MembershipStatus::Left;
        membership
    }

    pub fn fake_admin_group_membership_from_id(invitation_id: i32) -> GroupMembership {
        let mut membership = Self::fake_group_membership_from_id(invitation_id);
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
            id: 1,
            role: Some(MemberRole::Admin),
            left_at: None,
            membership_status: None,
        }
    }

    pub fn fake_leave_group_membership() -> UpdateGroupMembership {
        UpdateGroupMembership {
            id: 1,
            role: None,
            left_at: Some(Utc::now()),
            membership_status: Some(MembershipStatus::Left),
        }
    }

    pub fn fake_create_group_membership_dto() -> GroupMembershipCreateDto {
        GroupMembershipCreateDto {
            invitation_id: 1,
        }
    }

    pub fn fake_create_admin_group_membership_dto() -> CreateAdminGroupMembershipDto {
        CreateAdminGroupMembershipDto {
            invitation_id: 1,
        }
    }

    pub fn fake_leave_group_membership_dto() -> LeaveGroupMembershipDto {
        LeaveGroupMembershipDto {
            id: 1
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
            invitation_id: 1,
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
            invitation_id: 1,
        }
    }

    pub fn fake_group_membership_with_invitation_row_from_membership_and_invitation(membership: &GroupMembership, invitation: &Invitation) -> GroupMembershipWithInvitationRow {
        GroupMembershipWithInvitationRow {
            id: membership.id,
            user_id: invitation.to_user_id,
            group_chat_id: invitation.group_chat_id,
            role: membership.role.clone(),
            joined_at: membership.joined_at,
            left_at: membership.left_at,
            membership_status: membership.membership_status.clone(),
            invitation_id: membership.invitation_id,
        }
    }

    pub fn fake_group_membership_with_invitation_row() -> GroupMembershipWithInvitationRow {
        GroupMembershipWithInvitationRow {
            id: 1,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: 100,
            user_id: 42,
            group_chat_id: 77,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fake_new_group_membership() {
        let membership = GroupMembershipFactory::fake_new_group_membership();
        assert_eq!(membership.invitation_id, 1);
        assert_eq!(membership.role, MemberRole::Member);
    }

    #[test]
    fn test_fake_new_admin_group_membership() {
        let membership = GroupMembershipFactory::fake_new_admin_group_membership();
        assert_eq!(membership.invitation_id, 1);
        assert_eq!(membership.role, MemberRole::Admin);
    }

    #[test]
    fn test_fake_group_membership() {
        let membership = GroupMembershipFactory::fake_group_membership();
        assert_eq!(membership.id, 1);
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
