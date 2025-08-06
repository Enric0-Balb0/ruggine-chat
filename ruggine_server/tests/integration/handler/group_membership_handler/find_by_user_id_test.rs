use ruggine_server::handler::group_membership_handler::find_by_user_id::find_by_user_id;
use ruggine_server::state::group_membership_state::GroupMembershipState;
use ruggine_server::entity::group_membership::{MemberRole, MembershipStatus};
use axum::{extract::State, Extension};
use crate::common::{
    cleanup_user, cleanup_group_chat, cleanup_invitation, cleanup_group_membership,
    create_test_user, create_test_group_chat, create_test_invitation, create_test_group_membership
};
use crate::get_database;

#[cfg(test)]
mod find_by_user_id_handler_integration_tests {
    use super::*;

    /// Helper function to create a real group membership state with database connections
    async fn create_group_membership_state() -> GroupMembershipState {
        let db = get_database().await;
        GroupMembershipState::new(&db)
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_handler_finds_single_membership_successfully() {
        // Arrange: Create users, group chat, invitation, and membership
        let (admin_user, _) = create_test_user("find_by_user_id_admin_single").await;
        let (member_user, _) = create_test_user("find_by_user_id_member_single").await;
        let group_chat = create_test_group_chat("find_by_user_id_single", admin_user.id).await;
        let invitation = create_test_invitation(admin_user.id, member_user.id, group_chat.id).await;
        let membership = create_test_group_membership(invitation.id, member_user.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        
        // Act: Call find_by_user_id handler
        let result = find_by_user_id(
            Extension(member_user.clone()),
            State(group_membership_state),
        ).await;

        // Assert: Should succeed and return single membership
        assert!(result.is_ok(), "Find user memberships should succeed");
        let memberships_response = result.unwrap().0;
        let memberships = memberships_response.data();
        
        assert_eq!(memberships.len(), 1);
        let membership_dto = &memberships[0];
        
        assert_eq!(membership_dto.id, membership.id);
        assert_eq!(membership_dto.user_id, member_user.id);
        assert_eq!(membership_dto.group_chat_id, group_chat.id);
        assert_eq!(membership_dto.invitation_id, invitation.id);
        assert_eq!(membership_dto.role, membership.role);
        assert_eq!(membership_dto.membership_status, membership.membership_status);

        // Cleanup
        cleanup_group_membership(membership.id).await;
        cleanup_invitation(invitation.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(member_user.email).await;
        cleanup_user(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_handler_finds_multiple_memberships_successfully() {
        // Arrange: Create users and multiple group memberships
        let (admin_user, _) = create_test_user("find_by_user_id_admin_multi").await;
        let (member_user, _) = create_test_user("find_by_user_id_member_multi").await;
        
        // Create multiple group chats and memberships
        let group_chat1 = create_test_group_chat("find_by_user_id_multi1", admin_user.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_multi2", admin_user.id).await;
        let group_chat3 = create_test_group_chat("find_by_user_id_multi3", admin_user.id).await;
        
        let invitation1 = create_test_invitation(admin_user.id, member_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(admin_user.id, member_user.id, group_chat2.id).await;
        let invitation3 = create_test_invitation(admin_user.id, member_user.id, group_chat3.id).await;
        
        let membership1 = create_test_group_membership(invitation1.id, member_user.id).await;
        let membership2 = create_test_group_membership(invitation2.id, member_user.id).await;
        let membership3 = create_test_group_membership(invitation3.id, member_user.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        
        // Act: Call find_by_user_id handler
        let result = find_by_user_id(
            Extension(member_user.clone()),
            State(group_membership_state),
        ).await;

        // Assert: Should succeed and return all memberships
        assert!(result.is_ok(), "Find user memberships should succeed for multiple memberships");
        let memberships_response = result.unwrap().0;
        let memberships = memberships_response.data();
        
        assert_eq!(memberships.len(), 3);
        
        // Verify all memberships belong to the user
        for membership_dto in memberships {
            assert_eq!(membership_dto.user_id, member_user.id);
            assert_eq!(membership_dto.membership_status, MembershipStatus::Active);
        }
        
        // Verify we have all the expected group chat IDs
        let group_chat_ids: std::collections::HashSet<_> = memberships.iter().map(|m| m.group_chat_id).collect();
        assert!(group_chat_ids.contains(&group_chat1.id));
        assert!(group_chat_ids.contains(&group_chat2.id));
        assert!(group_chat_ids.contains(&group_chat3.id));
        
        // Verify ordering by joined_at DESC (newest first)
        for i in 1..memberships.len() {
            assert!(memberships[i-1].joined_at >= memberships[i].joined_at, 
                    "Memberships should be ordered by joined_at DESC");
        }

        // Cleanup
        cleanup_group_membership(membership1.id).await;
        cleanup_group_membership(membership2.id).await;
        cleanup_group_membership(membership3.id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_invitation(invitation3.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_group_chat(group_chat3.id).await;
        cleanup_user(member_user.email).await;
        cleanup_user(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_handler_returns_empty_for_user_with_no_memberships() {
        // Arrange: Create user with no memberships
        let (user, _) = create_test_user("find_by_user_id_no_memberships").await;
        
        let group_membership_state = create_group_membership_state().await;
        
        // Act: Call find_by_user_id handler
        let result = find_by_user_id(
            Extension(user.clone()),
            State(group_membership_state),
        ).await;

        // Assert: Should succeed and return empty vector
        assert!(result.is_ok(), "Find user memberships should succeed even with no memberships");
        let memberships_response = result.unwrap().0;
        let memberships = memberships_response.data();
        
        assert_eq!(memberships.len(), 0);

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_handler_user_isolation() {
        // Arrange: Create multiple users with separate memberships to test isolation
        let (admin_user, _) = create_test_user("find_by_user_id_admin_isolation").await;
        let (member_user1, _) = create_test_user("find_by_user_id_member1_isolation").await;
        let (member_user2, _) = create_test_user("find_by_user_id_member2_isolation").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_isolation1", admin_user.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_isolation2", admin_user.id).await;
        
        let invitation1 = create_test_invitation(admin_user.id, member_user1.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(admin_user.id, member_user2.id, group_chat2.id).await;
        
        let membership1 = create_test_group_membership(invitation1.id, member_user1.id).await;
        let membership2 = create_test_group_membership(invitation2.id, member_user2.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        
        // Act: Call find_by_user_id handler for each user separately
        let result1 = find_by_user_id(
            Extension(member_user1.clone()),
            State(group_membership_state.clone()),
        ).await;
        
        let result2 = find_by_user_id(
            Extension(member_user2.clone()),
            State(group_membership_state),
        ).await;

        // Assert: Each user should only see their own memberships
        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let binding1 = result1.unwrap();
        let memberships1 = binding1.0.data();
        let binding2 = result2.unwrap();
        let memberships2 = binding2.0.data();
        
        assert_eq!(memberships1.len(), 1);
        assert_eq!(memberships2.len(), 1);
        
        // Verify user 1 only sees their membership
        assert_eq!(memberships1[0].id, membership1.id);
        assert_eq!(memberships1[0].user_id, member_user1.id);
        assert_eq!(memberships1[0].group_chat_id, group_chat1.id);
        
        // Verify user 2 only sees their membership
        assert_eq!(memberships2[0].id, membership2.id);
        assert_eq!(memberships2[0].user_id, member_user2.id);
        assert_eq!(memberships2[0].group_chat_id, group_chat2.id);

        // Cleanup
        cleanup_group_membership(membership1.id).await;
        cleanup_group_membership(membership2.id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(member_user1.email).await;
        cleanup_user(member_user2.email).await;
        cleanup_user(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_handler_with_different_membership_roles() {
        // Arrange: Create user with memberships of different roles
        let (admin_user, _) = create_test_user("find_by_user_id_admin_roles").await;
        let (member_user, _) = create_test_user("find_by_user_id_member_roles").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_roles1", admin_user.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_roles2", admin_user.id).await;
        
        let invitation1 = create_test_invitation(admin_user.id, member_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(admin_user.id, member_user.id, group_chat2.id).await;
        
        // Create memberships (both will be Member role due to test helper limitations)
        let membership1 = create_test_group_membership(invitation1.id, member_user.id).await;
        let membership2 = create_test_group_membership(invitation2.id, member_user.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        
        // Act: Call find_by_user_id handler
        let result = find_by_user_id(
            Extension(member_user.clone()),
            State(group_membership_state),
        ).await;

        // Assert: Should find both memberships
        assert!(result.is_ok());
        let memberships_response = result.unwrap().0;
        let memberships = memberships_response.data();
        
        assert_eq!(memberships.len(), 2);
        
        // Verify both memberships have correct properties
        for membership_dto in memberships {
            assert_eq!(membership_dto.user_id, member_user.id);
            assert_eq!(membership_dto.membership_status, MembershipStatus::Active);
            // Both will be Member role due to test helper limitation
            assert_eq!(membership_dto.role, MemberRole::Member);
        }

        // Cleanup
        cleanup_group_membership(membership1.id).await;
        cleanup_group_membership(membership2.id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(member_user.email).await;
        cleanup_user(admin_user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_user_id_handler_ordering_by_join_date() {
        // Arrange: Create user with memberships at different times to test ordering
        let (admin_user, _) = create_test_user("find_by_user_id_admin_ordering").await;
        let (member_user, _) = create_test_user("find_by_user_id_member_ordering").await;
        
        let group_chat1 = create_test_group_chat("find_by_user_id_ordering1", admin_user.id).await;
        let group_chat2 = create_test_group_chat("find_by_user_id_ordering2", admin_user.id).await;
        
        let invitation1 = create_test_invitation(admin_user.id, member_user.id, group_chat1.id).await;
        let invitation2 = create_test_invitation(admin_user.id, member_user.id, group_chat2.id).await;
        
        // Create first membership
        let membership1 = create_test_group_membership(invitation1.id, member_user.id).await;
        
        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        // Create second membership
        let membership2 = create_test_group_membership(invitation2.id, member_user.id).await;
        
        let group_membership_state = create_group_membership_state().await;
        
        // Act: Call find_by_user_id handler
        let result = find_by_user_id(
            Extension(member_user.clone()),
            State(group_membership_state),
        ).await;

        // Assert: Should return memberships ordered by joined_at DESC
        assert!(result.is_ok());
        let memberships_response = result.unwrap().0;
        let memberships = memberships_response.data();
        
        assert_eq!(memberships.len(), 2);
        
        // Verify ordering - newer membership should be first
        assert!(memberships[0].joined_at >= memberships[1].joined_at, 
                "Memberships should be ordered by joined_at DESC");

        // Cleanup
        cleanup_group_membership(membership1.id).await;
        cleanup_group_membership(membership2.id).await;
        cleanup_invitation(invitation1.id).await;
        cleanup_invitation(invitation2.id).await;
        cleanup_group_chat(group_chat1.id).await;
        cleanup_group_chat(group_chat2.id).await;
        cleanup_user(member_user.email).await;
        cleanup_user(admin_user.email).await;
    }
}
