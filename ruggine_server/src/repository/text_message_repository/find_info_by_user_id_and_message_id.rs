use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::text_message::TextMessageInfo;
use crate::repository::text_message_repository::TextMessageRepository;

impl TextMessageRepository {
    pub async fn find_info_by_user_id_and_message_id_inner(&self, user_id: i32, text_message_id: i32) -> Result<TextMessageInfo, Error> {
        let rec = sqlx::query_as!(
            TextMessageInfo,
            r#"
            SELECT id, user_id, text_message_id, sent_at, read_at
            FROM text_message_info
            WHERE user_id = $1 AND text_message_id = $2
            "#,
            user_id,
            text_message_id
        )
            .fetch_one(self.db_conn.get_pool())
            .await?;

        Ok(rec)
    }
}



#[cfg(test)]
mod text_message_repository_find_by_user_id_and_message_id_tests {
    use mockall::predicate::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::repository::text_message_repository::TextMessageRepositoryTrait;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let user_id = 123i32;
        let message_id = 123i32;
        let expected_message = TextMessageFactory::fake_text_message_info_with_ids(1, user_id, message_id);

        mock_repo
            .expect_find_info_by_user_id_and_message_id()
            .with(eq(user_id), eq(message_id))
            .times(1)
            .returning(move |_, _| Box::pin({
                let value = expected_message.clone();
                async move { Ok(value.clone()) }
            }));

        // Act
        let result = mock_repo.find_info_by_user_id_and_message_id(user_id, message_id).await;

        // Assert
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.text_message_id, message_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_not_found() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let message_id = 999i32;
        let user_id = 123i32;
        mock_repo
            .expect_find_info_by_user_id_and_message_id()
            .with(eq(user_id), eq(message_id))
            .times(1)
            .returning(|_, _| Box::pin(async move {
                Err(sqlx::Error::RowNotFound)
            }));

        // Act
        let result = mock_repo.find_info_by_user_id_and_message_id(user_id, message_id).await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), sqlx::Error::RowNotFound);
    }
    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_database_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let message_id = 123i32;
        let user_id = 123i32;

        mock_repo
            .expect_find_info_by_user_id_and_message_id()
            .with(eq(user_id), eq(message_id))
            .times(1)
            .returning(|_, _| Box::pin(async move {
                Err(sqlx::Error::Configuration("Database connection failed".into()))
            }));

        // Act
        let result = mock_repo.find_info_by_user_id_and_message_id(user_id, message_id).await;

        // Assert
        assert!(result.is_err(), "Expected database error");
        match result.unwrap_err() {
            sqlx::Error::Configuration(_) => (), // ✅ Expected
            _ => panic!("Expected Configuration error"),
        }
    }
}
