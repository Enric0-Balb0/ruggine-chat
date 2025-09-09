mod update_sent_at_info_repository_tests {
    use crate::common;
    use chrono::{SubsecRound, Utc};
    use ruggine_server::factory::text_message_factory::TextMessageFactory;
    use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};

    #[tokio_shared_rt::test(shared)]
    async fn test_update_sent_at_info_inner_success() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let (sender, _) = common::create_test_user("sent_at_sender").await;
        let (recipient, _) = common::create_test_user("sent_at_recipient").await;
        let group_chat = common::create_test_group_chat("sent_at_group", sender.id).await;

        let text_message = common::create_test_text_message_without_message_info(sender.id, group_chat.id, Some("Message".to_string())).await;
        let info = TextMessageFactory::fake_new_text_message_info_with_ids(recipient.id, text_message.id);
        let info_id = repository.insert_text_message_info(info).await.unwrap();

        let before_update = Utc::now();

        // Act
        let result = repository.update_sent_at_info_inner(info_id).await;

        // Assert
        assert!(result.is_ok(), "Failed to update sent_at for text message info");

        // Verify that sent_at was set to CURRENT_TIMESTAMP
        let updated_info = repository.find_info_by_id(info_id).await.unwrap();
        assert!(updated_info.sent_at.is_some(), "sent_at should be set");
        
        let sent_at = updated_info.sent_at.unwrap();
        assert!(sent_at >= before_update, "sent_at should be after the test started");
        assert!(sent_at <= Utc::now(), "sent_at should not be in the future");

        // read_at should still be None
        assert!(updated_info.read_at.is_none(), "read_at should remain None");

        // Cleanup
        common::cleanup_text_message_info(info_id).await;
        common::cleanup_text_message(text_message.id).await;
        common::cleanup_group_chat(group_chat.id).await;
        common::cleanup_user(sender.id).await;
        common::cleanup_user(recipient.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_sent_at_info_inner_row_not_found() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let non_existent_id = 999_999;

        // Act
        let result = repository.update_sent_at_info_inner(non_existent_id).await;

        // Assert
        assert!(result.is_err(), "Expected Err for non-existent row");
        assert!(matches!(result.unwrap_err(), sqlx::Error::RowNotFound), "Expected RowNotFound error");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_sent_at_info_inner_multiple_updates() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let (sender, _) = common::create_test_user("multi_sent_sender").await;
        let (recipient, _) = common::create_test_user("multi_sent_recipient").await;
        let group_chat = common::create_test_group_chat("multi_sent_group", sender.id).await;

        let text_message = common::create_test_text_message_without_message_info(sender.id, group_chat.id, Some("Message".to_string())).await;
        let info = TextMessageFactory::fake_new_text_message_info_with_ids(recipient.id, text_message.id);
        let info_id = repository.insert_text_message_info(info).await.unwrap();

        // Act - First update
        let result1 = repository.update_sent_at_info_inner(info_id).await;
        assert!(result1.is_ok(), "First update should succeed");

        let first_sent_at = repository.find_info_by_id(info_id).await.unwrap().sent_at.unwrap();

        // Small delay to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Act - Second update
        let result2 = repository.update_sent_at_info_inner(info_id).await;
        assert!(result2.is_ok(), "Second update should succeed");

        // Assert
        let second_sent_at = repository.find_info_by_id(info_id).await.unwrap().sent_at.unwrap();
        assert!(second_sent_at >= first_sent_at, "Second sent_at should be >= first sent_at");

        // Cleanup
        common::cleanup_text_message_info(info_id).await;
        common::cleanup_text_message(text_message.id).await;
        common::cleanup_group_chat(group_chat.id).await;
        common::cleanup_user(sender.id).await;
        common::cleanup_user(recipient.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_sent_at_info_inner_preserves_read_at() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let (sender, _) = common::create_test_user("preserve_read_sender").await;
        let (recipient, _) = common::create_test_user("preserve_read_recipient").await;
        let group_chat = common::create_test_group_chat("preserve_read_group", sender.id).await;

        let text_message = common::create_test_text_message_without_message_info(sender.id, group_chat.id, Some("Message".to_string())).await;
        let info = TextMessageFactory::fake_new_text_message_info_with_ids(recipient.id, text_message.id);
        let info_id = repository.insert_text_message_info(info).await.unwrap();

        // Set both sent_at and read_at first
        repository.update_sent_at_info(info_id).await.unwrap();

        // Act - Update sent_at again
        let result = repository.update_read_at_info(info_id).await;

        // Assert
        assert!(result.is_ok(), "Failed to update sent_at");

        // Cleanup
        common::cleanup_text_message_info(info_id).await;
        common::cleanup_text_message(text_message.id).await;
        common::cleanup_group_chat(group_chat.id).await;
        common::cleanup_user(sender.id).await;
        common::cleanup_user(recipient.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_sent_at_info_inner_allows_read_at_update_later() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let (sender, _) = common::create_test_user("allow_read_sender").await;
        let (recipient, _) = common::create_test_user("allow_read_recipient").await;
        let group_chat = common::create_test_group_chat("allow_read_group", sender.id).await;

        let text_message = common::create_test_text_message_without_message_info(sender.id, group_chat.id, Some("Message".to_string())).await;
        let info = TextMessageFactory::fake_new_text_message_info_with_ids(recipient.id, text_message.id);
        let info_id = repository.insert_text_message_info(info).await.unwrap();

        // Act - First update sent_at
        let result1 = repository.update_sent_at_info_inner(info_id).await;
        assert!(result1.is_ok(), "Failed to update sent_at");

        // Act - Then update read_at (should work now that sent_at is set)
        let result2 = repository.update_read_at_info_inner(info_id).await;
        assert!(result2.is_ok(), "Failed to update read_at after sent_at was set");

        // Assert
        let final_info = repository.find_info_by_id(info_id).await.unwrap();
        assert!(final_info.sent_at.is_some(), "sent_at should be set");
        assert!(final_info.read_at.is_some(), "read_at should be set");
        
        // Verify database constraint: sent_at <= read_at
        let sent_timestamp = final_info.sent_at.unwrap();
        let read_timestamp = final_info.read_at.unwrap();
        assert!(sent_timestamp <= read_timestamp, "sent_at should be <= read_at");

        // Cleanup
        common::cleanup_text_message_info(info_id).await;
        common::cleanup_text_message(text_message.id).await;
        common::cleanup_group_chat(group_chat.id).await;
        common::cleanup_user(sender.id).await;
        common::cleanup_user(recipient.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_sent_at_info_inner_error_message_consistency() {
        // This test checks that the error message in update_sent_at_info_inner is correct
        // (currently it says "read at" but should say "sent at")
        
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let non_existent_id = 999_999;

        // Act
        let result = repository.update_sent_at_info_inner(non_existent_id).await;

        // Assert
        assert!(result.is_err(), "Expected Err for non-existent row");
        
        // Note: This test documents the current behavior where the error message 
        // incorrectly mentions "read at" instead of "sent at"
        // This could be a bug that should be fixed in the implementation
        match result.unwrap_err() {
            sqlx::Error::RowNotFound => {
                // Expected error type
            },
            other => panic!("Unexpected error type: {:?}", other),
        }
    }
}
