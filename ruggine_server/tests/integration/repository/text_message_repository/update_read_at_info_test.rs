mod update_read_at_info_repository_tests {
    use crate::common;
    use chrono::{SubsecRound, Utc};
    use ruggine_server::factory::text_message_factory::TextMessageFactory;
    use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};

    #[tokio_shared_rt::test(shared)]
    async fn test_update_read_at_info_inner_success() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let (sender, _) = common::create_test_user("read_at_sender").await;
        let (recipient, _) = common::create_test_user("read_at_recipient").await;
        let group_chat = common::create_test_group_chat("read_at_group", sender.id).await;

        let text_message = common::create_test_text_message_without_message_info(sender.id, group_chat.id, Some("Message".to_string())).await;
        let info = TextMessageFactory::fake_new_text_message_info_with_ids(recipient.id, text_message.id);
        let info_id = repository.insert_text_message_info(info).await.unwrap();

        // First set sent_at to ensure read_at can be updated (database constraint)
        repository.update_sent_at_info(info_id).await.unwrap();

        let before_update = Utc::now();

        // Act
        let result = repository.update_read_at_info_inner(info_id).await;

        // Assert
        assert!(result.is_ok(), "Failed to update read_at for text message info");

        // Verify that read_at was set to CURRENT_TIMESTAMP
        let updated_info = repository.find_info_by_id(info_id).await.unwrap();
        assert!(updated_info.read_at.is_some(), "read_at should be set");
        
        let read_at = updated_info.read_at.unwrap();
        assert!(read_at >= before_update, "read_at should be after the test started");
        assert!(read_at <= Utc::now(), "read_at should not be in the future");

        // Cleanup
        common::cleanup_text_message_info(info_id).await;
        common::cleanup_text_message(text_message.id).await;
        common::cleanup_group_chat(group_chat.id).await;
        common::cleanup_user(sender.id).await;
        common::cleanup_user(recipient.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_read_at_info_inner_row_not_found() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let non_existent_id = 999_999;

        // Act
        let result = repository.update_read_at_info_inner(non_existent_id).await;

        // Assert
        assert!(result.is_err(), "Expected Err for non-existent row");
        assert!(matches!(result.unwrap_err(), sqlx::Error::RowNotFound), "Expected RowNotFound error");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_read_at_info_inner_multiple_updates() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let (sender, _) = common::create_test_user("multi_read_sender").await;
        let (recipient, _) = common::create_test_user("multi_read_recipient").await;
        let group_chat = common::create_test_group_chat("multi_read_group", sender.id).await;

        let text_message = common::create_test_text_message_without_message_info(sender.id, group_chat.id, Some("Message".to_string())).await;
        let info = TextMessageFactory::fake_new_text_message_info_with_ids(recipient.id, text_message.id);
        let info_id = repository.insert_text_message_info(info).await.unwrap();

        // First set sent_at
        repository.update_sent_at_info(info_id).await.unwrap();

        // Act - First update
        let result1 = repository.update_read_at_info_inner(info_id).await;
        assert!(result1.is_ok(), "First update should succeed");

        let first_read_at = repository.find_info_by_id(info_id).await.unwrap().read_at.unwrap();

        // Small delay to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Act - Second update
        let result2 = repository.update_read_at_info_inner(info_id).await;
        assert!(result2.is_ok(), "Second update should succeed");

        // Assert
        let second_read_at = repository.find_info_by_id(info_id).await.unwrap().read_at.unwrap();
        assert!(second_read_at >= first_read_at, "Second read_at should be >= first read_at");

        // Cleanup
        common::cleanup_text_message_info(info_id).await;
        common::cleanup_text_message(text_message.id).await;
        common::cleanup_group_chat(group_chat.id).await;
        common::cleanup_user(sender.id).await;
        common::cleanup_user(recipient.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_read_at_info_inner_with_null_sent_at_fails() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let (sender, _) = common::create_test_user("null_sent_sender").await;
        let (recipient, _) = common::create_test_user("null_sent_recipient").await;
        let group_chat = common::create_test_group_chat("null_sent_group", sender.id).await;

        let text_message = common::create_test_text_message_without_message_info(sender.id, group_chat.id, Some("Message".to_string())).await;
        let info = TextMessageFactory::fake_new_text_message_info_with_ids(recipient.id, text_message.id);
        let info_id = repository.insert_text_message_info(info).await.unwrap();

        // Don't set sent_at, leave it as NULL

        // Act - Try to update read_at when sent_at is NULL (should fail due to database constraint)
        let result = repository.update_read_at_info_inner(info_id).await;

        // Assert
        assert!(result.is_err(), "Updating read_at when sent_at is NULL should fail");

        // Cleanup
        common::cleanup_text_message_info(info_id).await;
        common::cleanup_text_message(text_message.id).await;
        common::cleanup_group_chat(group_chat.id).await;
        common::cleanup_user(sender.id).await;
        common::cleanup_user(recipient.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_read_at_info_inner_preserves_sent_at() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let (sender, _) = common::create_test_user("preserve_sender").await;
        let (recipient, _) = common::create_test_user("preserve_recipient").await;
        let group_chat = common::create_test_group_chat("preserve_group", sender.id).await;

        let text_message = common::create_test_text_message_without_message_info(sender.id, group_chat.id, Some("Message".to_string())).await;
        let info = TextMessageFactory::fake_new_text_message_info_with_ids(recipient.id, text_message.id);
        let info_id = repository.insert_text_message_info(info).await.unwrap();

        // Set sent_at first
        repository.update_sent_at_info(info_id).await.unwrap();

        let sent_at = repository.find_info_by_id(info_id).await.unwrap().sent_at.unwrap().trunc_subsecs(6);

        // Act - Update read_at
        let result = repository.update_read_at_info_inner(info_id).await;

        // Assert
        assert!(result.is_ok(), "Failed to update read_at");

        let updated_info = repository.find_info_by_id(info_id).await.unwrap();
        assert!(updated_info.read_at.is_some(), "read_at should be set");
        assert!(updated_info.sent_at.is_some(), "sent_at should be preserved");
        
        // Verify sent_at wasn't changed
        let preserved_sent_at = updated_info.sent_at.unwrap().timestamp_micros();
        let expected_sent_at = sent_at.timestamp_micros();
        assert_eq!(preserved_sent_at, expected_sent_at, "sent_at should be preserved");

        // Cleanup
        common::cleanup_text_message_info(info_id).await;
        common::cleanup_text_message(text_message.id).await;
        common::cleanup_group_chat(group_chat.id).await;
        common::cleanup_user(sender.id).await;
        common::cleanup_user(recipient.id).await;
    }
}
