// Integration tests for Invitation API

#[cfg(test)]
mod invitation_api_integration_tests {
    use crate::common::*;
    use ruggine_client_ui::types::invitation::{MemberRole, MembershipStatus};

    #[tokio::test]
    async fn test_member_role_variants() {
        // Test that we can create both role variants
        let admin_membership = GroupFactory::admin_group_membership(1, 1);
        let member_membership = GroupFactory::member_group_membership(2, 1);
        
        assert!(matches!(admin_membership.role, MemberRole::Admin));
        assert!(matches!(member_membership.role, MemberRole::Member));
    }
    
    #[tokio::test]
    async fn test_member_role_equality() {
        let admin1 = MemberRole::Admin;
        let admin2 = MemberRole::Admin;
        let member = MemberRole::Member;
        
        assert_eq!(admin1, admin2);
        assert_ne!(admin1, member);
    }
    
    #[tokio::test]
    async fn test_membership_status_variants() {
        // Verify MembershipStatus variants exist and are usable
        let _pending = MembershipStatus::Pending;
        let _active = MembershipStatus::Active;
        let _inactive = MembershipStatus::Inactive;
        
        // Test they can be compared
        assert_ne!(MembershipStatus::Pending, MembershipStatus::Active);
        assert_ne!(MembershipStatus::Active, MembershipStatus::Inactive);
        assert_eq!(MembershipStatus::Pending, MembershipStatus::Pending);
    }
    
    #[tokio::test]
    async fn test_role_serialization_consistency() {
        // Test that roles maintain their identity through creation
        let admin_membership = GroupFactory::admin_group_membership(5, 10);
        let member_membership = GroupFactory::member_group_membership(5, 10);
        
        // Both should be for the same user and group
        assert_eq!(admin_membership.user_id, member_membership.user_id);
        assert_eq!(admin_membership.group_id, member_membership.group_id);
        
        // But different roles
        assert_ne!(admin_membership.role, member_membership.role);
    }
    
    #[tokio::test]
    async fn test_group_membership_with_details_roles() {
        let admin_details = GroupFactory::group_membership_with_details(
            1, 1, "Admin Group", MemberRole::Admin
        );
        let member_details = GroupFactory::group_membership_with_details(
            2, 1, "Member Group", MemberRole::Member
        );
        
        assert!(matches!(admin_details.membership.role, MemberRole::Admin));
        assert!(matches!(member_details.membership.role, MemberRole::Member));
        
        assert_eq!(admin_details.group_details.name, "Admin Group");
        assert_eq!(member_details.group_details.name, "Member Group");
    }
    
    #[tokio::test]
    async fn test_multiple_memberships_different_roles() {
        let memberships = vec![
            GroupFactory::admin_group_membership(1, 1),
            GroupFactory::member_group_membership(1, 2),
            GroupFactory::admin_group_membership(1, 3),
            GroupFactory::member_group_membership(1, 4),
        ];
        
        // User 1 should have different roles in different groups
        assert!(matches!(memberships[0].role, MemberRole::Admin));
        assert!(matches!(memberships[1].role, MemberRole::Member));
        assert!(matches!(memberships[2].role, MemberRole::Admin));
        assert!(matches!(memberships[3].role, MemberRole::Member));
        
        // All for the same user
        for membership in &memberships {
            assert_eq!(membership.user_id, 1);
        }
        
        // Different groups
        assert_eq!(memberships[0].group_id, 1);
        assert_eq!(memberships[1].group_id, 2);
        assert_eq!(memberships[2].group_id, 3);
        assert_eq!(memberships[3].group_id, 4);
    }
    
    #[tokio::test]
    async fn test_membership_timestamps() {
        let membership = GroupFactory::mock_group_membership("timestamp_test");
        
        assert!(!membership.joined_at.to_string().is_empty());
        
        // Create multiple memberships and ensure they have realistic timestamps
        let membership1 = GroupFactory::admin_group_membership(1, 1);
        let membership2 = GroupFactory::member_group_membership(1, 2);
        
        // Both should have valid timestamps (though they might be very close)
        assert!(!membership1.joined_at.to_string().is_empty());
        assert!(!membership2.joined_at.to_string().is_empty());
    }
    
    #[tokio::test]
    async fn test_role_consistency_in_details() {
        let admin_details = GroupFactory::group_membership_with_details(
            1, 42, "Test Group", MemberRole::Admin
        );
        
        // The membership and group details should be consistent
        assert_eq!(admin_details.membership.group_id, admin_details.group_details.id);
        assert_eq!(admin_details.group_details.id, 42);
        assert_eq!(admin_details.group_details.name, "Test Group");
        assert!(matches!(admin_details.membership.role, MemberRole::Admin));
    }
}
