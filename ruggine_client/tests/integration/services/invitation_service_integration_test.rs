// Integration tests for Invitation Service

use std::sync::Mutex;

#[cfg(test)]
mod invitation_service_integration_tests {
    use super::*;
    use crate::common::*;
    use ruggine_client_ui::api::services::invitation::InvitationService;
    use ruggine_client_ui::api::client::ApiClient;
    use ruggine_client_ui::utils::storage::StorageService;
    use ruggine_client_ui::types::invitation::{MemberRole, MembershipStatus};

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_invitation_service() -> InvitationService {
        let api_client = ApiClient::new("http://localhost:3000".to_string());
        let storage_service = StorageService::new();
        InvitationService::new(api_client, storage_service)
    }

    #[tokio::test]
    async fn test_invitation_service_creation() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_invitation_service();
        
        // Service should be created successfully
        drop(service);
    }

    #[tokio::test]
    async fn test_member_role_handling() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_invitation_service();
        
        // Test different member roles
        let admin_membership = GroupFactory::admin_group_membership(1, 1);
        let member_membership = GroupFactory::member_group_membership(2, 1);
        
        assert!(matches!(admin_membership.role, MemberRole::Admin));
        assert!(matches!(member_membership.role, MemberRole::Member));
        
        // Verify role equality and inequality
        assert_eq!(admin_membership.role, MemberRole::Admin);
        assert_eq!(member_membership.role, MemberRole::Member);
        assert_ne!(admin_membership.role, member_membership.role);
    }

    #[tokio::test]
    async fn test_membership_status_variants() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_invitation_service();
        
    // Test all MembershipStatus variants present in the current types
    let active = MembershipStatus::Active;
    let left = MembershipStatus::Left;
    let banned = MembershipStatus::Banned;

    // Test equality
    assert_eq!(active, MembershipStatus::Active);
    assert_eq!(left, MembershipStatus::Left);
    assert_eq!(banned, MembershipStatus::Banned);

    // Test inequality
    assert_ne!(active, left);
    assert_ne!(left, banned);
    assert_ne!(active, banned);
    }

    #[tokio::test]
    async fn test_invitation_service_role_management() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_invitation_service();
        
        // Create multiple memberships with different roles
        let memberships = vec![
            GroupFactory::admin_group_membership(1, 1),
            GroupFactory::member_group_membership(1, 2),
            GroupFactory::admin_group_membership(2, 1),
            GroupFactory::member_group_membership(2, 2),
        ];
        
        // User 1 should have different roles in different groups
        assert!(matches!(memberships[0].role, MemberRole::Admin));  // User 1, Group 1
        assert!(matches!(memberships[1].role, MemberRole::Member)); // User 1, Group 2
        
        // User 2 should have different roles in different groups
        assert!(matches!(memberships[2].role, MemberRole::Admin));  // User 2, Group 1
        assert!(matches!(memberships[3].role, MemberRole::Member)); // User 2, Group 2
        
    // Verify user and group assignments
    assert_eq!(memberships[0].user_id, 1);
    assert_eq!(memberships[0].group_chat_id, 1);
    assert_eq!(memberships[1].user_id, 1);
    assert_eq!(memberships[1].group_chat_id, 2);
    }

    #[tokio::test]
    async fn test_invitation_service_membership_with_details() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_invitation_service();
        
    // Create membership and group details separately (current factories provide these separately)
    let admin_membership = GroupFactory::membership_with_details(1, 42, MemberRole::Admin);
    let admin_group = GroupFactory::group_with_details(42, "Admin Group", 1);

    let member_membership = GroupFactory::membership_with_details(2, 42, MemberRole::Member);
    let member_group = GroupFactory::group_with_details(42, "Member Group", 1);

    // Verify admin details
    assert_eq!(admin_membership.user_id, 1);
    assert_eq!(admin_membership.group_chat_id, 42);
    assert_eq!(admin_group.id, 42);
    assert_eq!(admin_group.name, "Admin Group");
    assert!(matches!(admin_membership.role, MemberRole::Admin));

    // Verify member details
    assert_eq!(member_membership.user_id, 2);
    assert_eq!(member_membership.group_chat_id, 42);
    assert_eq!(member_group.id, 42);
    assert_eq!(member_group.name, "Member Group");
    assert!(matches!(member_membership.role, MemberRole::Member));
    }

    #[tokio::test]
    async fn test_invitation_service_mock_data_consistency() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_invitation_service();
        
        // Test mock membership creation
        let membership1 = GroupFactory::mock_group_membership("test1");
        let membership2 = GroupFactory::mock_group_membership("test2");
        
        // Should have unique IDs
        assert_ne!(membership1.id, membership2.id);
        
        // Should have valid timestamps
        assert!(!membership1.joined_at.to_string().is_empty());
        assert!(!membership2.joined_at.to_string().is_empty());
        
    // Should have valid user and group IDs
    assert!(membership1.user_id > 0);
    assert!(membership1.group_chat_id > 0);
    assert!(membership2.user_id > 0);
    assert!(membership2.group_chat_id > 0);
    }

    #[tokio::test]
    async fn test_invitation_service_role_permissions_simulation() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_invitation_service();
        
        // Simulate different permission scenarios
        let group_admin = GroupFactory::admin_group_membership(1, 1);
        let group_member = GroupFactory::member_group_membership(2, 1);
        
        // In a real scenario, admin should have more permissions
        // Here we just verify the role distinction is maintained
        assert!(matches!(group_admin.role, MemberRole::Admin));
        assert!(matches!(group_member.role, MemberRole::Member));
        
    // Both should be in the same group
    assert_eq!(group_admin.group_chat_id, group_member.group_chat_id);
        
        // But different users
        assert_ne!(group_admin.user_id, group_member.user_id);
    }

    #[tokio::test]
    async fn test_invitation_service_group_membership_batch() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_invitation_service();
        
        // Create multiple group memberships for batch operations
        // Use pairs (membership, group_detail) because factories provide membership and group separately
        let group_memberships: Vec<_> = (1..=5)
            .map(|i| (
                GroupFactory::membership_with_details(
                    i,
                    1,
                    if i % 2 == 0 { MemberRole::Admin } else { MemberRole::Member }
                ),
                GroupFactory::group_with_details(1, "Test Group", 1),
            ))
            .collect();

        assert_eq!(group_memberships.len(), 5);

        // Verify alternating roles
        for (i, (membership, group_detail)) in group_memberships.iter().enumerate() {
            let user_id = (i + 1) as i32;
            assert_eq!(membership.user_id, user_id);
            assert_eq!(group_detail.name, "Test Group");

            if user_id % 2 == 0 {
                assert!(matches!(membership.role, MemberRole::Admin));
            } else {
                assert!(matches!(membership.role, MemberRole::Member));
            }
        }
    }

    #[tokio::test]
    async fn test_invitation_service_membership_timestamps() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_invitation_service();
        
        // Test that membership timestamps are valid
        let membership1 = GroupFactory::admin_group_membership(1, 1);
        let membership2 = GroupFactory::member_group_membership(1, 2);
        
        // Both should have valid join timestamps
        assert!(!membership1.joined_at.to_string().is_empty());
        assert!(!membership2.joined_at.to_string().is_empty());
        
        // Timestamps should be realistic (not in the future by much)
    let now_naive = chrono::Utc::now().naive_utc();
    // membership.joined_at may be chrono::DateTime or NaiveDateTime depending on factory; compare as naive
    let m1 = membership1.joined_at.naive_utc();
    let m2 = membership2.joined_at.naive_utc();
    assert!(m1 <= now_naive);
    assert!(m2 <= now_naive);
    }

    #[tokio::test]
    async fn test_invitation_service_edge_cases() {
        let _guard = TEST_MUTEX.lock().expect("Failed to acquire mutex");
        
        let service = setup_invitation_service();
        
        // Test edge cases for membership creation
        let same_user_different_groups = vec![
            GroupFactory::admin_group_membership(1, 1),
            GroupFactory::member_group_membership(1, 2),
            GroupFactory::admin_group_membership(1, 3),
        ];
        
        // Same user should be able to have different roles in different groups
        assert_eq!(same_user_different_groups[0].user_id, 1);
        assert_eq!(same_user_different_groups[1].user_id, 1);
        assert_eq!(same_user_different_groups[2].user_id, 1);
        
    assert_eq!(same_user_different_groups[0].group_chat_id, 1);
    assert_eq!(same_user_different_groups[1].group_chat_id, 2);
    assert_eq!(same_user_different_groups[2].group_chat_id, 3);
        
        assert!(matches!(same_user_different_groups[0].role, MemberRole::Admin));
        assert!(matches!(same_user_different_groups[1].role, MemberRole::Member));
        assert!(matches!(same_user_different_groups[2].role, MemberRole::Admin));
    }
}
