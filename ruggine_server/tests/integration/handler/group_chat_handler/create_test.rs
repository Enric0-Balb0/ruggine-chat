use ruggine_server::handler::group_chat_handler::create::create;
use ruggine_server::factory::group_chat_factory::GroupChatFactory;
use ruggine_server::error::request_error::ValidatedRequest;
use axum::{extract::State, Extension};
use crate::common::{create_test_user, cleanup_group_chat, cleanup_user, create_group_chat_state};

#[cfg(test)]
mod group_chat_create_handler_integration_tests {
    use ruggine_server::error::api_error::ApiError::DbError;
    use ruggine_server::factory::user_factory::UserFactory;
    use crate::{clean_up_group_membership_invitation_by_user_id_and_group_chat_id, create_login_and_get_token};
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_handler_success() {
        // Arrange: Create a real user and group chat state
        let group_chat_state = create_group_chat_state().await;
        let (user, _password, _token) = create_login_and_get_token("handler_create_success".to_string()).await;
        
        // Create unique group chat data using factory
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("handler_integration_success");

        // Act: Call the handler
        let result = create(
            Extension(user.clone()),
            State(group_chat_state),
            ValidatedRequest(create_dto.clone()),
        ).await;

        // Assert: Verify the group chat was created successfully
        assert!(result.is_ok(), "Group chat creation should succeed: {:?}", result);
        let response = result.unwrap().0;
        let group_dto = response.data();
        
        assert!(group_dto.id > 0, "Group ID should be positive");
        assert_eq!(group_dto.name, create_dto.name);
        assert_eq!(group_dto.description, create_dto.description);
        assert_eq!(group_dto.created_by, user.id);
        assert!(group_dto.created_at <= chrono::Utc::now());
        assert!(group_dto.updated_at <= chrono::Utc::now());

        // Cleanup: Delete the test group and user
        clean_up_group_membership_invitation_by_user_id_and_group_chat_id(user.id, group_dto.id).await;
        cleanup_group_chat(group_dto.id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_handler_with_empty_description() {
        // Arrange: Create a real user and group chat state
        let group_chat_state = create_group_chat_state().await;
        let (user, _password) = create_test_user("handler_empty_desc").await;
        
        // Create group chat data with empty description
        let mut create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("handler_empty_description");
        create_dto.description = String::new();

        // Act: Call the handler
        let result = create(
            Extension(user.clone()),
            State(group_chat_state),
            ValidatedRequest(create_dto.clone()),
        ).await;

        // Assert: Should succeed with empty description
        assert!(result.is_ok(), "Group chat creation with empty description should succeed");
        let response = result.unwrap().0;
        let group_dto = response.data();
        
        assert!(group_dto.id > 0, "Group ID should be positive");
        assert_eq!(group_dto.name, create_dto.name);
        assert_eq!(group_dto.description, String::new());
        assert_eq!(group_dto.created_by, user.id);

        // Cleanup: Delete the test group and user
        clean_up_group_membership_invitation_by_user_id_and_group_chat_id(user.id, group_dto.id).await;
        cleanup_group_chat(group_dto.id).await;
        cleanup_user(user.email).await;
    }

    /* #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_handler_with_invalid_payload_empty_name() {
        // Arrange: Create a real user and group chat state
        let group_chat_state = create_group_chat_state().await;
        let (user, _password) = create_test_user("handler_invalid_payload").await;
        
        // Create group chat data with empty name (should fail validation)
        let invalid_dto = GroupChatCreateDto {
            name: String::new(), // Empty name should fail validation
            description: "Valid description".to_string(),
        };

        // Act: Call the handler
        let result = create_group_chat(
            Extension(user.clone()),
            State(group_chat_state),
            ValidatedRequest(invalid_dto),
        ).await;

        // Assert: Should fail due to validation error
        assert!(result.is_err(), "Group chat creation with empty name should fail");

        // Cleanup: Delete the test user
        cleanup_user(user.email).await;
    } */

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_handler_with_nonexistent_user() {
        // Arrange: Create group chat state and fake user
        let group_chat_state = create_group_chat_state().await;
        
        // Create a fake user that doesn't exist in database
        let mut fake_user = UserFactory::fake_user();
        fake_user.id = -1;
        
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("handler_nonexistent_user");

        // Act: Call the handler with non-existent user
        let result = create(
            Extension(fake_user),
            State(group_chat_state),
            ValidatedRequest(create_dto),
        ).await;

        // Assert: Should fail with UserNotFound error
        assert!(result.is_err(), "Should fail for non-existent user");
        match result.unwrap_err() {
            DbError(_) => {
                // Expected error
            }
            other => panic!("Expected DbError error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_handler_multiple_groups_same_user() {
        // Arrange: Create a real user and group chat state
        let group_chat_state = create_group_chat_state().await;
        let (user, _password) = create_test_user("handler_multiple_groups").await;
        
        // Create multiple unique group chat DTOs
        let create_dto1 = GroupChatFactory::unique_fake_group_chat_create_dto("handler_multi_1");
        let create_dto2 = GroupChatFactory::unique_fake_group_chat_create_dto("handler_multi_2");
        let create_dto3 = GroupChatFactory::unique_fake_group_chat_create_dto("handler_multi_3");

        // Act: Create multiple groups for the same user
        let result1 = create(
            Extension(user.clone()),
            State(group_chat_state.clone()),
            ValidatedRequest(create_dto1.clone()),
        ).await;

        let result2 = create(
            Extension(user.clone()),
            State(group_chat_state.clone()),
            ValidatedRequest(create_dto2.clone()),
        ).await;

        let result3 = create(
            Extension(user.clone()),
            State(group_chat_state),
            ValidatedRequest(create_dto3.clone()),
        ).await;

        // Assert: All groups should be created successfully
        assert!(result1.is_ok(), "First group creation should succeed");
        assert!(result2.is_ok(), "Second group creation should succeed");
        assert!(result3.is_ok(), "Third group creation should succeed");

        let group1 = result1.unwrap().0.data().clone();
        let group2 = result2.unwrap().0.data().clone();
        let group3 = result3.unwrap().0.data().clone();

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

        // Cleanup: Delete all test groups and user
        clean_up_group_membership_invitation_by_user_id_and_group_chat_id(user.id, group1.id).await;
        clean_up_group_membership_invitation_by_user_id_and_group_chat_id(user.id, group2.id).await;
        clean_up_group_membership_invitation_by_user_id_and_group_chat_id(user.id, group3.id).await;
        cleanup_group_chat(group1.id).await;
        cleanup_group_chat(group2.id).await;
        cleanup_group_chat(group3.id).await;
        cleanup_user(user.email).await;
    }
}
