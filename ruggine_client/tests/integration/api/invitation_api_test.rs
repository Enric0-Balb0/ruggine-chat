// Integration tests for Invitation API

#[cfg(test)]
mod invitation_api_integration_tests {
    use crate::common::*;
    use ruggine_client_ui::types::invitation::{MemberRole, MembershipStatus};

    #[tokio::test]
    async fn test_member_role_variants_and_equality() {
        let admin1 = MemberRole::Admin;
        let admin2 = MemberRole::Admin;
        let member = MemberRole::Member;

        assert_eq!(admin1, admin2);
        assert_ne!(admin1, member);
    }

    #[tokio::test]
    async fn test_membership_status_variants() {
        // Verify MembershipStatus variants exist and are usable
        let _active = MembershipStatus::Active;
        let _left = MembershipStatus::Left;
        let _banned = MembershipStatus::Banned;

        assert_ne!(MembershipStatus::Active, MembershipStatus::Left);
        assert_ne!(MembershipStatus::Left, MembershipStatus::Banned);
    }

    #[tokio::test]
    async fn test_role_serialization_consistency() {
        let admin_membership = GroupFactory::admin_group_membership(5, 10);
        let member_membership = GroupFactory::member_group_membership(5, 10);

        // Both should be for the same user and group_chat_id
        assert_eq!(admin_membership.user_id, member_membership.user_id);
        assert_eq!(admin_membership.group_chat_id, member_membership.group_chat_id);
        assert_ne!(admin_membership.role, member_membership.role);
    }

    #[tokio::test]
    async fn test_membership_with_details_and_group() {
        let membership = GroupFactory::membership_with_details(1, 1, MemberRole::Admin);
        let group = GroupFactory::group_with_details(1, "Admin Group", 1);

        assert!(matches!(membership.role, MemberRole::Admin));
        assert_eq!(group.name, "Admin Group");
        // Consistency: membership.group_chat_id should match group.id
        assert_eq!(membership.group_chat_id, group.id);
    }

    #[tokio::test]
    async fn test_multiple_memberships_different_roles_and_timestamps() {
        let memberships = vec![
            GroupFactory::admin_group_membership(1, 1),
            GroupFactory::member_group_membership(1, 2),
            GroupFactory::admin_group_membership(1, 3),
            GroupFactory::member_group_membership(1, 4),
        ];

        // User 1 should have different roles in different groups
        assert!(matches!(memberships[0].role, MemberRole::Admin));
        assert!(matches!(memberships[1].role, MemberRole::Member));

        for m in &memberships {
            assert_eq!(m.user_id, 1);
            assert!(!m.joined_at.to_string().is_empty());
        }

        assert_eq!(memberships[0].group_chat_id, 1);
        assert_eq!(memberships[1].group_chat_id, 2);
        assert_eq!(memberships[2].group_chat_id, 3);
        assert_eq!(memberships[3].group_chat_id, 4);
    }
}
