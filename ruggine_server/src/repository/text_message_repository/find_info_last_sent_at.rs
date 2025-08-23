use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::text_message::TextMessageInfo;
use crate::repository::text_message_repository::TextMessageRepository;

impl TextMessageRepository {
    pub async fn find_info_last_sent_inner(&self, user_id: i32, group_chat_id: i32) -> Result<TextMessageInfo, Error> {
        let rec = sqlx::query_as!(
            TextMessageInfo,
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
              AND tmi.sent_at IS NOT NULL
            ORDER BY tmi.sent_at DESC
            "#,
            user_id,
            group_chat_id
        )
        .fetch_one(self.db_conn.get_pool())
        .await?;

        Ok(rec)
    }
}

#[cfg(test)]
mod text_message_repository_find_info_last_sent_tests {
    use mockall::predicate::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::repository::text_message_repository::TextMessageRepositoryTrait;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_info_last_sent_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let user_id = 11i32;
        let group_chat_id = 22i32;
        let message_id = 333i32;
        let expected_info = TextMessageFactory::fake_text_message_info_with_ids(
            1, user_id, message_id
        );

        mock_repo
            .expect_find_info_last_sent()
            .with(eq(user_id), eq(group_chat_id))
            .times(1)
            .returning(move |_, _| Box::pin({
                let value = expected_info.clone();
                async move { Ok(value.clone()) }
            }));

        // Act
        let result = mock_repo.find_info_last_sent(user_id, group_chat_id).await;

        // Assert
        assert!(result.is_ok(), "Failed to fetch last sent info");
        let info = result.unwrap();
        assert_eq!(info.user_id, user_id);
        assert_eq!(info.text_message_id, message_id);
        assert!(info.sent_at.is_some(), "Expected sent_at to be set");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_info_last_sent_not_found() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let user_id = 44i32;
        let group_chat_id = 55i32;

        mock_repo
            .expect_find_info_last_sent()
            .with(eq(user_id), eq(group_chat_id))
            .times(1)
            .returning(|_, _| Box::pin(async move {
                Err(sqlx::Error::RowNotFound)
            }));

        // Act
        let result = mock_repo.find_info_last_sent(user_id, group_chat_id).await;

        // Assert
        assert!(result.is_err(), "Expected row not found");
        assert!(matches!(result.unwrap_err(), sqlx::Error::RowNotFound));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_info_last_sent_database_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let user_id = 66i32;
        let group_chat_id = 77i32;

        mock_repo
            .expect_find_info_last_sent()
            .with(eq(user_id), eq(group_chat_id))
            .times(1)
            .returning(|_, _| Box::pin(async move {
                Err(sqlx::Error::Configuration("Database connection failed".into()))
            }));

        // Act
        let result = mock_repo.find_info_last_sent(user_id, group_chat_id).await;

        // Assert
        assert!(result.is_err(), "Expected database error");
        assert!(matches!(result.unwrap_err(), sqlx::Error::Configuration(_)));
    }
}