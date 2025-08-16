use ruggine_server::handler::text_message_handler::create::create;
use ruggine_server::factory::text_message_factory::TextMessageFactory;
use ruggine_server::error::request_error::ValidatedRequest;
use axum::{extract::State, Extension};
use crate::common::{create_test_user, cleanup_user, cleanup_text_message, cleanup_group_chat, cleanup_test_user_from_a_group_chat, create_text_message_state, create_test_group_chat_with_invitation_and_membership};

#[cfg(test)]
mod text_message_create_handler_integration_tests {
    use ruggine_server::error::api_error::ApiError;
    use ruggine_server::error::text_message_error::TextMessageError;
    use crate::leave_user_from_a_group;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_handler_success() {
        // Arrange: Create a real user and text message state
        let text_message_state = create_text_message_state().await;
        let (user, _password) = create_test_user("handler_create_msg_success").await;
        
        // Create a test group with invitation and membership for the user
        let group_chat = create_test_group_chat_with_invitation_and_membership(
            "handler_create_msg_group",
            user.id,
        ).await;
        
        // Create unique text message data using factory
        let create_dto = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat.id);

        // Act: Call the handler
        let result = create(
            Extension(user.clone()),
            State(text_message_state),
            ValidatedRequest(create_dto.clone()),
        ).await;

        // Assert: Verify the text message was created successfully
        assert!(result.is_ok(), "Text message creation should succeed: {:?}", result);
        let response = result.unwrap().0;
        let message_dto = response.data();
        
        assert!(message_dto.id > 0, "Message ID should be positive");
        assert_eq!(message_dto.content, create_dto.content);
        assert_eq!(message_dto.sender_id, user.id);
        assert_eq!(message_dto.group_chat_id, group_chat.id);
        assert!(message_dto.sent_at <= chrono::Utc::now());

        // Cleanup: Delete the test message, group, and user
        cleanup_text_message(message_dto.id).await;
        cleanup_test_user_from_a_group_chat(user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_handler_with_long_content() {
        // Arrange: Create a real user and text message state
        let text_message_state = create_text_message_state().await;
        let (user, _password) = create_test_user("handler_long_msg").await;
        
        // Create a test group with invitation and membership for the user
        let group_chat = create_test_group_chat_with_invitation_and_membership(
            "handler_long_msg_group",
            user.id,
        ).await;
        
        // Create text message data with long content
        let mut create_dto = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat.id);
        create_dto.content = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(50);

        // Act: Call the handler
        let result = create(
            Extension(user.clone()),
            State(text_message_state),
            ValidatedRequest(create_dto.clone()),
        ).await;

        // Assert: Should succeed with long content
        assert!(result.is_ok(), "Text message creation with long content should succeed");
        let response = result.unwrap().0;
        let message_dto = response.data();
        
        assert!(message_dto.id > 0, "Message ID should be positive");
        assert_eq!(message_dto.content, create_dto.content);
        assert_eq!(message_dto.sender_id, user.id);
        assert_eq!(message_dto.group_chat_id, group_chat.id);

        // Cleanup: Delete the test message, group, and user
        cleanup_text_message(message_dto.id).await;
        cleanup_test_user_from_a_group_chat(user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_handler_user_not_member() {
        // Arrange: Create a real user and text message state
        let text_message_state = create_text_message_state().await;
        let (sender, _password) = create_test_user("handler_not_member").await;
        let (group_owner, _password) = create_test_user("handler_group_owner").await;
        
        // Create a group where sender is not a member
        let group_chat = create_test_group_chat_with_invitation_and_membership(
            "handler_private_group",
            group_owner.id,
        ).await;
        
        // Create message data for group sender is not member of
        let create_dto = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat.id);

        // Act: Call the handler
        let result = create(
            Extension(sender.clone()),
            State(text_message_state),
            ValidatedRequest(create_dto),
        ).await;

        // Assert: Should fail with user cannot access messages error
        assert!(result.is_err(), "Text message creation should fail for non-member");
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)));

        // Cleanup: Delete the test group and users
        cleanup_test_user_from_a_group_chat(group_owner.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(sender.id).await;
        cleanup_user(group_owner.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_handler_user_not_active_member() {
        // Arrange: Create a real user and text message state
        let text_message_state = create_text_message_state().await;
        let (user, _password) = create_test_user("handler_create_msg_user_not_active_member").await;

        // Create a test group with invitation and membership for the user
        let group_chat = create_test_group_chat_with_invitation_and_membership(
            "handler_create_msg_group",
            user.id,
        ).await;

        // Leave group
        leave_user_from_a_group(user.id, group_chat.id).await;

        // Create unique text message data using factory
        let create_dto = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat.id);

        // Act: Call the handler
        let result = create(
            Extension(user.clone()),
            State(text_message_state),
            ValidatedRequest(create_dto.clone()),
        ).await;

        // Assert: Verify the text message was created successfully
        assert!(result.is_err(), "Text message should fail for not active user for the group: {:?}", result);
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)));

        cleanup_test_user_from_a_group_chat(user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user.id).await;
    }


    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_handler_nonexistent_group() {
        // Arrange: Create a real user and text message state
        let text_message_state = create_text_message_state().await;
        let (user, _password) = create_test_user("handler_no_group").await;
        
        // Create message data for nonexistent group
        let nonexistent_group_id = 99999;
        let create_dto = TextMessageFactory::fake_text_message_create_dto_with_group_id(nonexistent_group_id);

        // Act: Call the handler
        let result = create(
            Extension(user.clone()),
            State(text_message_state),
            ValidatedRequest(create_dto),
        ).await;

        // Assert: Should fail with user cannot access messages error
        assert!(result.is_err(), "Text message creation should fail for nonexistent group");
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)));

        // Cleanup: Delete the test user
        cleanup_user(user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_handler_with_unique_factory_data() {
        // Arrange: Create a real user and text message state
        let text_message_state = create_text_message_state().await;
        let (user, _password) = create_test_user("handler_unique_data").await;
        
        // Create a test group with invitation and membership for the user
        let group_chat = create_test_group_chat_with_invitation_and_membership(
            "handler_unique_data_group",
            user.id,
        ).await;
        
        // Create unique text message data using factory
        let create_dto = TextMessageFactory::unique_fake_text_message_create_dto("handler_integration", group_chat.id);

        // Act: Call the handler
        let result = create(
            Extension(user.clone()),
            State(text_message_state),
            ValidatedRequest(create_dto.clone()),
        ).await;

        // Assert: Verify the text message was created successfully with unique data
        assert!(result.is_ok(), "Text message creation with unique data should succeed");
        let response = result.unwrap().0;
        let message_dto = response.data();
        
        assert!(message_dto.id > 0, "Message ID should be positive");
        assert!(message_dto.content.contains("handler_integration"), "Content should contain unique prefix");
        assert_eq!(message_dto.sender_id, user.id);
        assert_eq!(message_dto.group_chat_id, group_chat.id);

        // Cleanup: Delete the test message, group, and user
        cleanup_text_message(message_dto.id).await;
        cleanup_test_user_from_a_group_chat(user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_text_message_handler_with_special_characters() {
        // Arrange: Create a real user and text message state
        let text_message_state = create_text_message_state().await;
        let (user, _password) = create_test_user("handler_special_chars").await;
        
        // Create a test group with invitation and membership for the user
        let group_chat = create_test_group_chat_with_invitation_and_membership(
            "handler_special_chars_group",
            user.id,
        ).await;
        
        // Create text message data with special characters
        let mut create_dto = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat.id);
        create_dto.content = "Message with émojis 🎉 and special chars: àáâãäå çčć".to_string();

        // Act: Call the handler
        let result = create(
            Extension(user.clone()),
            State(text_message_state),
            ValidatedRequest(create_dto.clone()),
        ).await;

        // Assert: Should succeed with special characters
        assert!(result.is_ok(), "Text message creation with special characters should succeed");
        let response = result.unwrap().0;
        let message_dto = response.data();
        
        assert!(message_dto.id > 0, "Message ID should be positive");
        assert_eq!(message_dto.content, create_dto.content);
        assert_eq!(message_dto.sender_id, user.id);
        assert_eq!(message_dto.group_chat_id, group_chat.id);

        // Cleanup: Delete the test message, group, and user
        cleanup_text_message(message_dto.id).await;
        cleanup_test_user_from_a_group_chat(user.id, group_chat.id).await;
        cleanup_group_chat(group_chat.id).await;
        cleanup_user(user.id).await;
    }
}
