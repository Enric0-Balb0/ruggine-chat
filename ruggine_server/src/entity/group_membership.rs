use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use utoipa::ToSchema;

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow, Default, PartialEq, Eq)]
pub struct GroupMembership {
    pub id: i32,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub membership_status: MembershipStatus,
    pub current_action: CurrentAction,
    pub invitation_id: i32, // Foreign key to Invitation
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NewGroupMembership {
    pub role: MemberRole,
    pub invitation_id: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateGroupMembership {
    pub id: i32,
    pub role: Option<MemberRole>,
    pub membership_status: Option<MembershipStatus>,
    pub left_at: Option<DateTime<Utc>>,
    pub current_action: Option<CurrentAction>
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema, Display)]
#[sqlx(type_name = "membership_status")]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum MembershipStatus {
    Active,
    Left,
    Banned,
}

impl Default for MembershipStatus {
    fn default() -> Self {
        MembershipStatus::Active
    }
}

impl sqlx::postgres::PgHasArrayType for MembershipStatus {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_membership_status")
    }
}

pub fn all_membership_statuses() -> Vec<MembershipStatus> {
    vec![MembershipStatus::Active, MembershipStatus::Left, MembershipStatus::Banned]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema, Display)]
#[sqlx(type_name = "member_role")]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum MemberRole {
    Member,
    Admin,
}

pub fn all_member_roles() -> Vec<MemberRole> {
    vec![MemberRole::Member, MemberRole::Admin]
}

impl Default for MemberRole {
    fn default() -> Self {
        MemberRole::Member
    }
}

impl UpdateGroupMembership {
    pub fn has_updates(&self) -> bool {
        self.role.is_some()
            || self.left_at.is_some()
            || self.membership_status.is_some()
            || self.current_action.is_some()
    }

    pub fn is_valid(&self) -> bool {
        match self.membership_status {
            Some(MembershipStatus::Left) => self.left_at.is_some(),
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_role_default() {
        assert_eq!(MemberRole::default(), MemberRole::Member);
    }

    #[test]
    fn test_all_member_roles() {
        let roles = all_member_roles();
        assert_eq!(roles.len(), 2);
        assert!(roles.contains(&MemberRole::Member));
        assert!(roles.contains(&MemberRole::Admin));
    }

    #[test]
    fn test_update_group_membership_has_updates() {
        let update = UpdateGroupMembership {
            id: 1,
            role: Some(MemberRole::Admin),
            left_at: None,
            membership_status: None,
            current_action: None,
        };
        assert!(update.has_updates());

        let update = UpdateGroupMembership {
            id: 1,
            role: None,
            left_at: Some(Utc::now()),
            membership_status: None,
            current_action: None,
        };
        assert!(update.has_updates());

        let update = UpdateGroupMembership {
            id: 1,
            role: None,
            left_at: None,
            membership_status: None,
            current_action: None,
        };
        assert!(!update.has_updates());

        let update = UpdateGroupMembership {
            id: 1,
            role: None,
            left_at: None,
            membership_status: Some(MembershipStatus::Active),
            current_action: None,
        };
        assert!(update.has_updates());

        let update = UpdateGroupMembership {
            id: 1,
            role: None,
            left_at: None,
            membership_status: None,
            current_action: Some(CurrentAction::Writing),
        };
        assert!(update.has_updates());
    }

    #[test]
    fn test_group_membership_default() {
        let membership = GroupMembership::default();
        assert_eq!(membership.id, 0);
        assert_eq!(membership.invitation_id, 0);
        assert_eq!(membership.role, MemberRole::Member);
        assert!(membership.left_at.is_none());
    }

    #[test]
    fn test_new_group_membership() {
        let new_membership = NewGroupMembership {
            invitation_id: 1,
            role: MemberRole::Admin,
        };
        assert_eq!(new_membership.invitation_id, 1);
        assert_eq!(new_membership.role, MemberRole::Admin);
    }

    #[test]
    fn test_update_group_membership() {
        let update = UpdateGroupMembership {
            id: 1,
            role: Some(MemberRole::Admin),
            left_at: Some(Utc::now()),
            membership_status: Some(MembershipStatus::Left),
            current_action: Some(CurrentAction::Waiting),
        };

        assert_eq!(update.id, 1);
        assert_eq!(update.role, Some(MemberRole::Admin));
        assert!(update.left_at.is_some());
        assert_eq!(update.membership_status, Some(MembershipStatus::Left));
        assert_eq!(update.current_action, Some(CurrentAction::Waiting));
    }

    #[test]
    fn test_member_role_serialization() {
        let member = MemberRole::Member;
        let admin = MemberRole::Admin;
        
        // Test that the enum can be serialized/deserialized
        let member_json = serde_json::to_string(&member).unwrap();
        let admin_json = serde_json::to_string(&admin).unwrap();
        
        assert_eq!(member_json, "\"member\"");
        assert_eq!(admin_json, "\"admin\"");
        
        let member_deserialized: MemberRole = serde_json::from_str(&member_json).unwrap();
        let admin_deserialized: MemberRole = serde_json::from_str(&admin_json).unwrap();
        
        assert_eq!(member_deserialized, MemberRole::Member);
        assert_eq!(admin_deserialized, MemberRole::Admin);
    }

    #[test]
    fn test_membership_status_serialization() {
        let active = MembershipStatus::Active;
        let left = MembershipStatus::Left;

        // Test that the enum can be serialized/deserialized
        let active_json = serde_json::to_string(&active).unwrap();
        let left_json = serde_json::to_string(&left).unwrap();

        assert_eq!(active_json, "\"active\"");
        assert_eq!(left_json, "\"left\"");

        let active_deserialized: MembershipStatus = serde_json::from_str(&active_json).unwrap();
        let left_deserialized: MembershipStatus = serde_json::from_str(&left_json).unwrap();

        assert_eq!(active_deserialized, MembershipStatus::Active);
        assert_eq!(left_deserialized, MembershipStatus::Left);
    }

    #[test]
    fn test_update_group_membership_is_valid() {
        // membership_status is Left, left_at is Some => valid
        let update = UpdateGroupMembership {
            id: 1,
            role: None,
            left_at: Some(Utc::now()),
            membership_status: Some(MembershipStatus::Left),
            current_action: None,
        };
        assert!(update.is_valid());

        // membership_status is Left, left_at is None => not valid
        let update = UpdateGroupMembership {
            id: 1,
            role: None,
            left_at: None,
            membership_status: Some(MembershipStatus::Left),
            current_action: None,
        };
        assert!(!update.is_valid());

        // membership_status is Active, left_at is None => valid
        let update = UpdateGroupMembership {
            id: 1,
            role: None,
            left_at: None,
            membership_status: Some(MembershipStatus::Active),
            current_action: None,
        };
        assert!(update.is_valid());

        // membership_status is None, left_at is None => valid
        let update = UpdateGroupMembership {
            id: 1,
            role: None,
            left_at: None,
            membership_status: None,
            current_action: None,
        };
        assert!(update.is_valid());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema, Display)]
#[sqlx(type_name = "current_action")]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CurrentAction {
    Waiting,
    Writing,
}

impl Default for CurrentAction {
    fn default() -> Self {
        CurrentAction::Waiting
    }
}
