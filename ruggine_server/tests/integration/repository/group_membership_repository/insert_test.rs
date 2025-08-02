use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::factory::group_membership_factory::GroupMembershipFactory;
use ruggine_server::entity::group_membership::MemberRole;

#[cfg(test)]
mod group_membership_repository_integration_tests {
    use ruggine_server::entity::group_membership::MembershipStatus;
    use crate::{get_database, create_test_user, create_test_group_chat, cleanup_group_chat, cleanup_user, cleanup_group_membership};

    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_member_membership() {
        // Arrange
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        // Create test user and group chat for the membership
        let (user, _password) = create_test_user("membership_insert").await;
        let group_chat = create_test_group_chat("membership_insert", user.id).await;

        let new_membership = GroupMembershipFactory::fake_new_group_membership_with_ids(user.id, group_chat.id);

        // Act
        let result = repository.insert(new_membership.clone()).await;

        // Assert
        assert!(result.is_ok(), "Failed to insert group membership");
        let membership_id = result.unwrap();
        assert!(membership_id > 0, "Membership ID should be positive");

        // Verify the membership was actually inserted
        let found_membership = repository.find_by_id(membership_id).await;
        assert!(found_membership.is_ok(), "Failed to find inserted membership");
        
        let membership = found_membership.unwrap();
        assert_eq!(membership.user_id, new_membership.user_id);
        assert_eq!(membership.group_chat_id, new_membership.group_chat_id);
        assert_eq!(membership.role, new_membership.role);
        assert!(membership.left_at.is_none(), "New membership should not have left_at set");
        assert_eq!(membership.membership_status, MembershipStatus::Active, "New membership should have active status");

        // Cleanup
        cleanup_group_membership(membership_id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_admin_membership() {
        // Arrange
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        // Create test user and group chat for the membership
        let (user, _password) = create_test_user("admin_membership_insert").await;
        let group_chat = create_test_group_chat("admin_membership_insert", user.id).await;

        let new_membership = GroupMembershipFactory::fake_new_admin_group_membership_with_ids(user.id, group_chat.id);

        // Act
        let result = repository.insert(new_membership.clone()).await;

        // Assert
        assert!(result.is_ok(), "Failed to insert admin group membership");
        let membership_id = result.unwrap();
        assert!(membership_id > 0, "Membership ID should be positive");

        // Verify the membership was actually inserted with admin role
        let found_membership = repository.find_by_id(membership_id).await;
        assert!(found_membership.is_ok(), "Failed to find inserted admin membership");
        
        let membership = found_membership.unwrap();
        assert_eq!(membership.user_id, new_membership.user_id);
        assert_eq!(membership.group_chat_id, new_membership.group_chat_id);
        assert_eq!(membership.role, MemberRole::Admin);
        assert!(membership.left_at.is_none(), "New membership should not have left_at set");
        assert!(membership.membership_status == MembershipStatus::Active, "New membership should have active status");

        // Cleanup
        cleanup_group_membership(membership_id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_duplicate_membership() {
        // Arrange
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        // Create test user and group chat for the membership
        let (user, _password) = create_test_user("duplicate_membership").await;
        let group_chat = create_test_group_chat("duplicate_membership", user.id).await;

        let membership1 = GroupMembershipFactory::fake_new_group_membership_with_ids(user.id, group_chat.id);
        let membership2 = GroupMembershipFactory::fake_new_group_membership_with_ids(user.id, group_chat.id);

        // Act
        let first_insert = repository.insert(membership1.clone()).await;
        let second_insert = repository.insert(membership2.clone()).await;

        // Assert
        assert!(first_insert.is_ok(), "First membership insert should succeed");
        // Second insert should fail due to unique constraint (user can't have multiple active memberships in same group)
        assert!(second_insert.is_err(), "Second membership insert should fail due to unique constraint");

        // Cleanup
        if let Ok(membership_id) = first_insert {
            cleanup_group_membership(membership_id).await;
        }
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_multiple_users_same_group() {
        // Arrange
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        // Create test users and group chat
        let (user1, _password1) = create_test_user("multi_user1").await;
        let (user2, _password2) = create_test_user("multi_user2").await;
        let group_chat = create_test_group_chat("multi_user_group", user1.id).await;

        let membership1 = GroupMembershipFactory::fake_new_group_membership_with_ids(user1.id, group_chat.id);
        let membership2 = GroupMembershipFactory::fake_new_group_membership_with_ids(user2.id, group_chat.id);

        // Act
        let result1 = repository.insert(membership1.clone()).await;
        let result2 = repository.insert(membership2.clone()).await;

        // Assert
        assert!(result1.is_ok(), "First user membership insert should succeed");
        assert!(result2.is_ok(), "Second user membership insert should succeed");

        let membership_id1 = result1.unwrap();
        let membership_id2 = result2.unwrap();
        assert_ne!(membership_id1, membership_id2, "Different memberships should have different IDs");

        // Verify both memberships exist
        let found_membership1 = repository.find_by_id(membership_id1).await;
        let found_membership2 = repository.find_by_id(membership_id2).await;

        assert!(found_membership1.is_ok(), "Failed to find first membership");
        assert!(found_membership2.is_ok(), "Failed to find second membership");

        // Cleanup
        cleanup_group_membership(membership_id1).await;
        cleanup_group_membership(membership_id2).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user1.email.clone()).await;
        cleanup_user(user2.email.clone()).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_foreign_key_constraint() {
        // Arrange
        let db = get_database().await;
        let repository = GroupMembershipRepository::new(&db);

        // Create membership with non-existent user_id and group_chat_id
        let (user, _) = create_test_user("fk_constraint").await;
        let test_group_chat = create_test_group_chat("fk_constraint", user.id).await;
        let invalid_membership = GroupMembershipFactory::fake_new_group_membership_with_ids(-1, -1);
        let invalid_membership_user_id = GroupMembershipFactory::fake_new_group_membership_with_ids(-1, test_group_chat.id);
        let invalid_membership_group_chat_id = GroupMembershipFactory::fake_new_group_membership_with_ids(user.id, -1);

        // Act
        let result = repository.insert(invalid_membership).await;
        let result_user_id = repository.insert(invalid_membership_user_id).await;
        let result_group_chat_id = repository.insert(invalid_membership_group_chat_id).await;

        // Assert
        assert!(result.is_err(), "Insert should fail due to foreign key constraint");
        assert!(result_user_id.is_err(), "Insert should fail due to foreign key constraint");
        assert!(result_group_chat_id.is_err(), "Insert should fail due to foreign key constraint");

        // Verify it's a database constraint error
        match result.unwrap_err() {
            sqlx::Error::Database(db_err) => {
                // PostgreSQL foreign key violation error code is "23503"
                assert_eq!(db_err.code().as_deref(), Some("23503"), "Error code should be 23503");
            }
            _ => panic!("Expected database constraint error"),
        }
        match result_user_id.unwrap_err() {
            sqlx::Error::Database(db_err) => {
                assert_eq!(db_err.code().as_deref(), Some("23503"), "Error code should be 23503");
            }
            _ => panic!("Expected database constraint error"),
        }
        match result_group_chat_id.unwrap_err() {
            sqlx::Error::Database(db_err) => {
                assert_eq!(db_err.code().as_deref(), Some("23503"), "Error code should be 23503");
            }
            _ => panic!("Expected database constraint error"),
            
        }

        // Cleanup
        cleanup_group_chat(test_group_chat.id).await;
        cleanup_user(user.email.clone()).await;
    }
}
