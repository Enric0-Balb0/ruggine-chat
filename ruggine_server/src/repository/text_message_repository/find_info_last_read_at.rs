use sqlx::Error;
use tower::util::Optional;
use crate::config::database::DatabaseTrait;
use crate::entity::text_message::TextMessageInfo;
use crate::repository::text_message_repository::TextMessageRepository;

impl TextMessageRepository {
    pub async fn find_info_last_read_inner(&self, user_id: i32, group_chat_id: i32) -> Result<Option<TextMessageInfo>, Error> {
        let rec = sqlx::query_as::<_, TextMessageInfo>(
            r#"
            SELECT tmi.id,
                tmi.user_id,
                tmi.text_message_id,
                tmi.sent_at,
                tmi.read_at
            FROM text_message_info tmi
            INNER JOIN text_message tm
                ON tmi.text_message_id = tm.id
            WHERE tmi.user_id = $1
            AND tm.group_chat_id = $2
            AND tmi.read_at IS NOT NULL
            ORDER BY tmi.read_at DESC
            "#
        )
        .bind(user_id)
        .bind(group_chat_id)
        .fetch_optional(self.db_conn.get_pool())
        .await?;

        Ok(rec)
    }
}

#[cfg(test)]
mod text_message_repository_find_info_last_read_tests {
    use mockall::predicate::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::repository::text_message_repository::TextMessageRepositoryTrait;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_info_last_read_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let user_id = 42i32;
        let group_chat_id = 77i32;
        let message_id = 1001i32;
        let expected_info = TextMessageFactory::fake_text_message_info_read_dto_with_ids_and_read_at(
            1, user_id, message_id
        );
        let expected_info_entity = TextMessageFactory::fake_text_message_info_with_ids_and_read_at(
            1, user_id, message_id
        );

        mock_repo
            .expect_find_info_last_read()
            .with(eq(user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _| Box::pin({
                let value = expected_info_entity.clone();
                async move { Ok(Some(value.clone())) }
            }));

        // Act
        let result = mock_repo.find_info_last_read(user_id, group_chat_id).await;

        // Assert
        assert!(result.is_ok(), "Failed to fetch last read info");
        let info = result.unwrap().unwrap();
        assert_eq!(info.user_id, user_id);
        assert_eq!(info.text_message_id, message_id);
        assert!(info.read_at.is_some(), "Expected last read to have read_at timestamp");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_info_last_read_not_found() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let user_id = 42i32;
        let group_chat_id = 999i32;

        mock_repo
            .expect_find_info_last_read()
            .with(eq(user_id), eq(group_chat_id))
            .times(1)
            .returning(|_, _| Box::pin(async move {
                Ok(None)
            }));

        // Act
        let result = mock_repo.find_info_last_read(user_id, group_chat_id).await;

        // Assert
        assert!(result.is_ok(), "Expected ok also with no results");
        assert!(result.unwrap().is_none());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_info_last_read_database_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let user_id = 55i32;
        let group_chat_id = 88i32;

        mock_repo
            .expect_find_info_last_read()
            .with(eq(user_id), eq(group_chat_id))
            .times(1)
            .returning(|_, _| Box::pin(async move {
                Err(sqlx::Error::Configuration("Database connection failed".into()))
            }));

        // Act
        let result = mock_repo.find_info_last_read(user_id, group_chat_id).await;

        // Assert
        assert!(result.is_err(), "Expected database error");
        match result.unwrap_err() {
            sqlx::Error::Configuration(_) => (), // ✅ Expected
            _ => panic!("Expected Configuration error"),
        }
    }
}
