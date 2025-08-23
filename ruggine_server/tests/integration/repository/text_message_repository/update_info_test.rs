mod update_info_repository_tests {
    use crate::common;
    use chrono::{SubsecRound, Utc};
    use ruggine_server::factory::text_message_factory::TextMessageFactory;
    use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};

    #[tokio_shared_rt::test(shared)]
    async fn test_update_info_inner_success_full_update() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let (sender, _) = common::create_test_user("update_full_sender").await;
        let (recipient, _) = common::create_test_user("update_full_recipient").await;
        let group_chat = common::create_test_group_chat("update_full_group", sender.id).await;

        let text_message = common::create_test_text_message(sender.id, group_chat.id, Some("Message".to_string())).await;
        let info = TextMessageFactory::fake_new_text_message_info_with_ids(recipient.id, text_message.id);
        let info_id = repository.insert_text_message_info(info).await.unwrap();

        let sent_at = Utc::now();
        let read_at = Utc::now();

        let update_info = ruggine_server::entity::text_message::TextMessageInfoUpdate {
            id: info_id,
            sent_at: Some(sent_at),
            read_at: Some(read_at),
        };

        // Act
        let result = repository.update_info_inner(update_info.clone()).await;

        // Assert
        assert!(result.is_ok(), "Failed to update text message info");

        // Verify updated values
        let updated_info = repository.find_info_by_id(info_id).await.unwrap();
        let updated_sent_at = updated_info.sent_at.unwrap().timestamp_micros();
        let expected_sent_at = sent_at.timestamp_micros();
        let updated_read_at = updated_info.read_at.unwrap().timestamp_micros();
        let expected_read_at = read_at.timestamp_micros();
        assert_eq!(updated_sent_at, expected_sent_at);
        assert_eq!(updated_read_at, expected_read_at);

        // Cleanup
        common::cleanup_text_message_info(info_id).await;
        common::cleanup_text_message(text_message.id).await;
        common::cleanup_group_chat(group_chat.id).await;
        common::cleanup_user(sender.id).await;
        common::cleanup_user(recipient.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_info_inner_partial_update() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let (sender, _) = common::create_test_user("update_partial_sender").await;
        let (recipient, _) = common::create_test_user("update_partial_recipient").await;
        let group_chat = common::create_test_group_chat("update_partial_group", sender.id).await;

        let text_message = common::create_test_text_message(sender.id, group_chat.id, Some("Message".to_string())).await;
        let info = TextMessageFactory::fake_new_text_message_info_with_ids(recipient.id, text_message.id);
        let info_id = repository.insert_text_message_info(info).await.unwrap();

        let sent_at = Utc::now();
        let update_info = ruggine_server::entity::text_message::TextMessageInfoUpdate {
            id: info_id,
            sent_at: Some(sent_at),
            read_at: None,
        };

        // Act
        let result = repository.update_info_inner(update_info.clone()).await;

        // Assert
        assert!(result.is_ok(), "Failed to partially update text message info");

        let updated_info = repository.find_info_by_id(info_id).await.unwrap();
        assert!(updated_info.read_at.is_none());
        let updated_sent_at = updated_info.sent_at.unwrap().timestamp_micros();
        let expected_sent_at = sent_at.timestamp_micros();
        assert_eq!(updated_sent_at, expected_sent_at);

        // Cleanup
        common::cleanup_text_message_info(info_id).await;
        common::cleanup_text_message(text_message.id).await;
        common::cleanup_group_chat(group_chat.id).await;
        common::cleanup_user(sender.id).await;
        common::cleanup_user(recipient.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_info_inner_nothing_to_update() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let (sender, _) = common::create_test_user("update_none_sender").await;
        let (recipient, _) = common::create_test_user("update_none_recipient").await;
        let group_chat = common::create_test_group_chat("update_none_group", sender.id).await;

        let text_message = common::create_test_text_message(sender.id, group_chat.id, Some("Message".to_string())).await;
        let info = TextMessageFactory::fake_new_text_message_info_with_ids(recipient.id, text_message.id);
        let info_id = repository.insert_text_message_info(info).await.unwrap();

        let update_info = ruggine_server::entity::text_message::TextMessageInfoUpdate {
            id: info_id,
            sent_at: None,
            read_at: None,
        };

        // Act
        let result = repository.update_info_inner(update_info.clone()).await;

        // Assert
        assert!(result.is_ok(), "Expected Ok when nothing to update");

        // Cleanup
        common::cleanup_text_message_info(info_id).await;
        common::cleanup_text_message(text_message.id).await;
        common::cleanup_group_chat(group_chat.id).await;
        common::cleanup_user(sender.id).await;
        common::cleanup_user(recipient.id).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_info_inner_row_not_found() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let update_info = ruggine_server::entity::text_message::TextMessageInfoUpdate {
            id: 999_999, // Non-existent ID
            sent_at: Some(Utc::now()),
            read_at: Some(Utc::now()),
        };

        // Act
        let result = repository.update_info_inner(update_info.clone()).await;

        // Assert
        assert!(result.is_err(), "Expected Err for non-existent row");
        assert!(matches!(result.unwrap_err(), sqlx::Error::RowNotFound));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_info_inner_read_at_trigger_logic() {
        // Arrange
        let db = common::get_database().await;
        let repository = TextMessageRepository::new(&db);

        let (sender, _) = common::create_test_user("trigger_sender").await;
        let (recipient, _) = common::create_test_user("trigger_recipient").await;
        let group_chat = common::create_test_group_chat("trigger_group", sender.id).await;

        let text_message = common::create_test_text_message(sender.id, group_chat.id, Some("Message".to_string())).await;
        let info = TextMessageFactory::fake_new_text_message_info_with_ids(recipient.id, text_message.id);
        let info_id = repository.insert_text_message_info(info).await.unwrap();

        let now = Utc::now().trunc_subsecs(6);

        // -----------------------------
        // ❌ Caso invalido 1: sent_at è NULL, read_at non può essere settato
        // -----------------------------
        let update_invalid_null_sent = ruggine_server::entity::text_message::TextMessageInfoUpdate {
            id: info_id,
            sent_at: None,
            read_at: Some(now + chrono::Duration::seconds(20)),
        };

        let result_invalid_null_sent = repository.update_info_inner(update_invalid_null_sent).await;
        assert!(result_invalid_null_sent.is_err(), "Update invalido con sent_at NULL dovrebbe fallire");

        // -----------------------------
        // ❌ Caso invalido 2: sent_at > read_at
        // -----------------------------
        let update_invalid_sent_gt_read = ruggine_server::entity::text_message::TextMessageInfoUpdate {
            id: info_id,
            sent_at: Some(now + chrono::Duration::seconds(30)),
            read_at: Some(now + chrono::Duration::seconds(20)),
        };

        let result_invalid_sent_gt_read = repository.update_info_inner(update_invalid_sent_gt_read).await;
        assert!(result_invalid_sent_gt_read.is_err(), "Update invalido con sent_at > read_at dovrebbe fallire");

        // Cleanup
        common::cleanup_text_message_info(info_id).await;
        common::cleanup_text_message(text_message.id).await;
        common::cleanup_group_chat(group_chat.id).await;
        common::cleanup_user(sender.id).await;
        common::cleanup_user(recipient.id).await;
    }
}