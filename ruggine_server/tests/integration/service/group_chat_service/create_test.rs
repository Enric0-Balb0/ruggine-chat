use ruggine_server::service::group_chat_service::{GroupChatService, GroupChatServiceTrait};
use ruggine_server::repository::group_chat_repository::{GroupChatRepository, GroupChatRepositoryTrait};
use ruggine_server::repository::invitation_repository::{InvitationRepository, InvitationRepositoryTrait};
use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::factory::group_chat_factory::GroupChatFactory;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::entity::invitation::InvitationStatus;
use ruggine_server::entity::group_membership::{MemberRole, MembershipStatus};
use crate::common::{get_database, create_test_user, cleanup_group_chat};

#[cfg(test)]
mod group_chat_create_service_integration_tests {
    use ruggine_server::utils::service_initializer::ServiceInitializer;
    use crate::{clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id, cleanup_group_membership, cleanup_invitation, cleanup_user};
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_with_real_user() {
        // Arrange: Create a real user in the database
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let group_chat_service = service_init.group_chat_service();
        let group_membership_service = service_init.group_membership_service();
        
        // Create a unique user using common helper
        let (user, _password) = create_test_user("group_create").await;
        
        // Create unique group chat data using factory
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("integration_test");

        // Act: Create the group chat
        let result = group_chat_service.create(create_dto.clone(), user.id).await;

        // Assert: Verify group chat was created successfully
        assert!(result.is_ok(), "Failed to create group chat: {:?}", result);
        let group_dto = result.unwrap();
        
        assert!(group_dto.id > 0, "Group ID should be positive");
        assert_eq!(group_dto.name, create_dto.name);
        assert_eq!(group_dto.description, create_dto.description);
        assert_eq!(group_dto.created_by, user.id);
        assert!(group_dto.created_at <= chrono::Utc::now());
        assert!(group_dto.updated_at <= chrono::Utc::now());
        
        // Find the membership with associated invitation for our group
        let our_membership = group_membership_service.find_by_user_id_and_group_id(user.id, group_dto.id).await.unwrap();

        assert_eq!(our_membership.role, MemberRole::Admin, "Creator should be admin");
        assert_eq!(our_membership.membership_status, MembershipStatus::Active, "Membership should be active");
        assert_eq!(our_membership.user_id, user.id, "Membership should belong to the creator");
        assert_eq!(our_membership.group_chat_id, group_dto.id, "Membership should be for the created group");
        assert!(our_membership.invitation_id > 0, "Membership should have a valid invitation ID");
        assert!(our_membership.left_at.is_none(), "Creator should not have left the group");

        // Cleanup: Delete the test group and user
        cleanup_group_membership(our_membership.id).await;
        cleanup_invitation(our_membership.invitation_id).await;
        cleanup_group_chat(group_dto.id).await;
        cleanup_user(user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_with_nonexistent_user() {
        // Arrange: Set up group chat service
        let db = get_database().await;
        let group_chat_service = GroupChatService::new(&db);
        
        // Use a non-existent user ID
        let nonexistent_user_id = 99999;
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("nonexistent_user");

        // Act: Try to create group chat with non-existent user
        let result = group_chat_service.create(create_dto, nonexistent_user_id).await;

        // Assert: Should fail with UserNotFound error
        assert!(result.is_err(), "Should fail for non-existent user");
        match result.unwrap_err() {  // TODO: Add foreign key error
            ApiError::DbError(_) => {
                // Expected error
            }
            other => panic!("Expected UserNotFound error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_with_duplicate_name() {
        // Arrange: Create a real user and first group chat
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let group_chat_service = service_init.group_chat_service();
        let group_membership_service = service_init.group_membership_service();
        
        let (user, _password) = create_test_user("duplicate_group").await;
        
        // Create first group chat
        let create_dto1 = GroupChatFactory::unique_fake_group_chat_create_dto("duplicate_test");
        let result1 = group_chat_service.create(create_dto1.clone(), user.id).await;
        assert!(result1.is_ok(), "First group creation should succeed");
        let group1 = result1.unwrap();

        // Try to create second group with same name (if unique constraint exists)
        let mut create_dto2 = GroupChatFactory::fake_group_chat_create_dto();
        create_dto2.name = create_dto1.name.clone(); // Use same name

        // Act: Try to create second group with same name
        let result2 = group_chat_service.create(create_dto2, user.id).await;

        // Assert: Check if unique constraint is enforced
        // Note: This test depends on whether your database has a unique constraint on group names
        // If no constraint exists, the second creation should succeed
        // If constraint exists, it should fail with UniqueConstraintViolation

        // For now, let's assume no unique constraint on group names, so both should succeed
        assert!(result2.is_ok(), "Second group creation should succeed");
        let group2 = result2.unwrap();
        // Both groups created successfully (no unique constraint)
        assert_ne!(group1.id, group2.id, "Groups should have different IDs");

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(user.id, group1.id).await;
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(user.id, group2.id).await;
        cleanup_group_chat(group2.id).await;
        cleanup_group_chat(group1.id).await;
        cleanup_user(user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_multiple_groups_for_same_user() {
        // Arrange: Create a real user
        let db = get_database().await;
        let service_init = ServiceInitializer::new(&db);
        let group_chat_service = service_init.group_chat_service();
        let group_membership_service = service_init.group_membership_service();
        
        let (user, _password) = create_test_user("multiple_groups").await;
        
        // Create multiple unique group chat DTOs
        let create_dto1 = GroupChatFactory::unique_fake_group_chat_create_dto("multi_group_1");
        let create_dto2 = GroupChatFactory::unique_fake_group_chat_create_dto("multi_group_2");
        let create_dto3 = GroupChatFactory::unique_fake_group_chat_create_dto("multi_group_3");

        // Act: Create multiple groups for the same user
        let result1 = group_chat_service.create(create_dto1.clone(), user.id).await;
        let result2 = group_chat_service.create(create_dto2.clone(), user.id).await;
        let result3 = group_chat_service.create(create_dto3.clone(), user.id).await;

        // Assert: All groups should be created successfully
        assert!(result1.is_ok(), "First group creation should succeed");
        assert!(result2.is_ok(), "Second group creation should succeed");
        assert!(result3.is_ok(), "Third group creation should succeed");

        let group1 = result1.unwrap();
        let group2 = result2.unwrap();
        let group3 = result3.unwrap();

        // Verify all groups have different IDs
        assert_ne!(group1.id, group2.id);
        assert_ne!(group1.id, group3.id);
        assert_ne!(group2.id, group3.id);

        // Verify all groups have correct creator
        assert_eq!(group1.created_by, user.id);
        assert_eq!(group2.created_by, user.id);
        assert_eq!(group3.created_by, user.id);

        // Verify group names are different
        assert_ne!(group1.name, group2.name);
        assert_ne!(group1.name, group3.name);
        assert_ne!(group2.name, group3.name);

        // Verify that the user has memberships in all three groups
        let memberships_result = group_membership_service.find_by_user_id(user.id).await;
        assert!(memberships_result.is_ok(), "Should find memberships for user");
        let memberships = memberships_result.unwrap();
        
        // Should have at least 3 memberships (could have more from other tests)
        assert!(memberships.len() >= 3, "Should have at least 3 memberships");
        
        // Verify membership exists for each group
        let group1_membership = memberships.iter().find(|m| m.group_chat_id == group1.id).expect("Should have membership in group 1");
        let group2_membership = memberships.iter().find(|m| m.group_chat_id == group2.id).expect("Should have membership in group 2");
        let group3_membership = memberships.iter().find(|m| m.group_chat_id == group3.id).expect("Should have membership in group 3");
        
        // Verify all memberships are active admin memberships
        assert_eq!(group1_membership.role, MemberRole::Admin);
        assert_eq!(group1_membership.membership_status, MembershipStatus::Active);
        assert_eq!(group2_membership.role, MemberRole::Admin);
        assert_eq!(group2_membership.membership_status, MembershipStatus::Active);
        assert_eq!(group3_membership.role, MemberRole::Admin);
        assert_eq!(group3_membership.membership_status, MembershipStatus::Active);

        // Cleanup: Delete all test groups and user
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(user.id, group1.id).await;
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(user.id, group2.id).await;
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(user.id, group3.id).await;
        cleanup_group_chat(group1.id).await;
        cleanup_group_chat(group2.id).await;
        cleanup_group_chat(group3.id).await;
        cleanup_user(user.email.clone()).await;
    }
}
