// Integration tests for Group API

#[cfg(test)]
mod group_api_integration_tests {
    use crate::common::*;
    use ruggine_client_ui::types::{
        group::{GroupChatCreateRequest, GroupChat},
        membership::GroupMembership,
        invitation::MemberRole,
    };

    #[tokio::test]
    async fn test_group_api_types_validation() {
        // Test GroupChatCreateRequest validation
        let valid_request = GroupFactory::basic_group_create_request();
        assert!(!valid_request.name.is_empty());
        assert!(valid_request.description.is_some());
        
        let minimal_request = GroupFactory::minimal_group_create_request("test");
        assert!(!minimal_request.name.is_empty());
    }
    
    #[tokio::test]
    async fn test_group_chat_creation_data() {
        let group = GroupFactory::mock_group_chat();
        
        assert!(group.id > 0);
        assert!(!group.name.is_empty());
        assert!(group.description.is_some());
        assert!(group.created_by > 0);
        assert!(!group.created_at.to_string().is_empty());
        assert!(!group.updated_at.to_string().is_empty());
    }
    
    #[tokio::test]
    async fn test_group_with_specific_details() {
        let group = GroupFactory::group_with_details(42, "Special Group", 1);
        
        assert_eq!(group.id, 42);
        assert_eq!(group.name, "Special Group");
        assert_eq!(group.created_by, 1);
        assert!(group.description.is_some());
    }
    
    #[tokio::test]
    async fn test_group_membership_creation() {
        let membership = GroupFactory::mock_group_membership("test");
        
        assert!(membership.id > 0);
        assert!(membership.user_id > 0);
        assert!(membership.group_id > 0);
        assert!(matches!(membership.role, MemberRole::Member | MemberRole::Admin));
    }
    
    #[tokio::test]
    async fn test_admin_group_membership() {
        let admin_membership = GroupFactory::admin_group_membership(1, 1);
        
        assert_eq!(admin_membership.user_id, 1);
        assert_eq!(admin_membership.group_id, 1);
        assert!(matches!(admin_membership.role, MemberRole::Admin));
    }
    
    #[tokio::test]
    async fn test_member_group_membership() {
        let member_membership = GroupFactory::member_group_membership(2, 1);
        
        assert_eq!(member_membership.user_id, 2);
        assert_eq!(member_membership.group_id, 1);
        assert!(matches!(member_membership.role, MemberRole::Member));
    }
    
    #[tokio::test]
    async fn test_group_membership_with_details() {
        let membership_details = GroupFactory::mock_group_membership_with_details();
        
        assert!(membership_details.membership.id > 0);
        assert!(membership_details.group_details.id > 0);
        assert!(!membership_details.group_details.name.is_empty());
    }
    
    #[tokio::test]
    async fn test_specific_group_membership_with_details() {
        let membership_details = GroupFactory::group_membership_with_details(
            1, 42, "Test Group", MemberRole::Admin
        );
        
        assert_eq!(membership_details.membership.user_id, 1);
        assert_eq!(membership_details.membership.group_id, 42);
        assert_eq!(membership_details.group_details.id, 42);
        assert_eq!(membership_details.group_details.name, "Test Group");
        assert!(matches!(membership_details.membership.role, MemberRole::Admin));
    }
    
    #[tokio::test]
    async fn test_group_name_uniqueness() {
        let group1 = GroupFactory::minimal_group_create_request("unique1");
        let group2 = GroupFactory::minimal_group_create_request("unique2");
        
        assert_ne!(group1.name, group2.name);
    }
    
    #[tokio::test]
    async fn test_multiple_groups_creation() {
        let groups: Vec<GroupChat> = (0..5)
            .map(|i| GroupFactory::group_with_details(i, &format!("Group {}", i), 1))
            .collect();
            
        assert_eq!(groups.len(), 5);
        
        // Verify all groups have unique IDs and names
        for (i, group) in groups.iter().enumerate() {
            assert_eq!(group.id, i as i32);
            assert_eq!(group.name, format!("Group {}", i));
        }
    }
}
