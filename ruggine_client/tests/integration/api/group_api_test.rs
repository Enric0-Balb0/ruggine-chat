// Integration tests for Group API (consolidated)

#[cfg(test)]
mod group_api_integration_tests {
    use crate::common::*;
    use ruggine_client_ui::types::{
        group::GroupChat,
        invitation::MemberRole,
    };

    #[tokio::test]
    async fn test_group_api_types_validation() {
        // Use a minimal create request available in factories
        let valid_request = GroupFactory::minimal_group_create_request("test");
        assert!(!valid_request.name.is_empty());
        // description is a String in current types
        assert!(!valid_request.description.is_empty());

        let minimal_request = GroupFactory::minimal_group_create_request("test2");
        assert!(!minimal_request.name.is_empty());
    }

    #[tokio::test]
    async fn test_group_chat_creation_data() {
        let group = GroupFactory::mock_group_chat();

        assert!(group.id > 0);
        assert!(!group.name.is_empty());
        assert!(!group.description.is_empty());
        assert!(group.created_by > 0);
        assert!(!group.created_at.to_string().is_empty());
        assert!(!group.updated_at.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_group_membership_and_details() {
        let membership = GroupFactory::mock_group_membership("test");
        assert!(membership.id > 0);
        assert!(membership.user_id > 0);
        assert!(membership.group_chat_id > 0);
        assert!(matches!(membership.role, MemberRole::Member | MemberRole::Admin));

        let admin = GroupFactory::admin_group_membership(1, 1);
        assert_eq!(admin.user_id, 1);
        assert_eq!(admin.group_chat_id, 1);

        let member = GroupFactory::member_group_membership(2, 1);
        assert_eq!(member.user_id, 2);
        assert_eq!(member.group_chat_id, 1);
    }

    #[tokio::test]
    async fn test_group_with_details_helpers() {
        let details = GroupFactory::membership_with_details(1, 42, MemberRole::Admin);
        let group = GroupFactory::group_with_details(42, "Test Group", 1);

        assert_eq!(details.group_chat_id, group.id);
        assert_eq!(group.name, "Test Group");
    }

    #[tokio::test]
    async fn test_group_name_uniqueness_and_multiple() {
        let group1 = GroupFactory::minimal_group_create_request("unique1");
        let group2 = GroupFactory::minimal_group_create_request("unique2");

        assert_ne!(group1.name, group2.name);

        let groups: Vec<GroupChat> = (0..5)
            .map(|i| GroupFactory::group_with_details(i as i32, &format!("Group {}", i), 1))
            .collect();

        assert_eq!(groups.len(), 5);
        for (i, group) in groups.iter().enumerate() {
            assert_eq!(group.id, i as i32);
            assert_eq!(group.name, format!("Group {}", i));
        }
    }
}
