use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::text_message::TextMessage;
use crate::repository::text_message_repository::TextMessageRepository;

impl TextMessageRepository {
    pub async fn find_inner(&self, id: i32) -> Result<TextMessage, Error> {
        let query = sqlx::query_as!(
            TextMessage,
            r#"
            SELECT id, content, sender_id, group_chat_id, sent_at
            FROM text_message
            WHERE id = $1
            "#,
            id
        );

        // se c'è una transazione, usala; altrimenti usa la pool
        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            query.fetch_one(&mut *tx_ref).await
        } else {
            query.fetch_one(self.db_conn.get_pool()).await
        }
    }
}

#[cfg(test)]
mod text_message_repository_find_by_id_tests {
    use mockall::predicate::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::repository::text_message_repository::TextMessageRepositoryTrait;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let message_id = 123i32;
        let expected_message = TextMessageFactory::fake_text_message_with_id(message_id);

        mock_repo
            .expect_find()
            .with(eq(message_id))
            .times(1)
            .returning(move |_| Box::pin({
                let value = expected_message.clone();
                async move { Ok(value.clone()) }
            }));

        // Act
        let result = mock_repo.find(message_id).await;

        // Assert
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message.id, message_id);
        assert_eq!(message.content, "Test message content");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_not_found() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let message_id = 999i32;

        mock_repo
            .expect_find()
            .with(eq(message_id))
            .times(1)
            .returning(|_| Box::pin(async move {
                Err(sqlx::Error::RowNotFound)
            }));

        // Act
        let result = mock_repo.find(message_id).await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), sqlx::Error::RowNotFound);
    }
    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_database_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let message_id = 123i32;

        mock_repo
            .expect_find()
            .with(eq(message_id))
            .times(1)
            .returning(|_| Box::pin(async move {
                Err(sqlx::Error::Configuration("Database connection failed".into()))
            }));

        // Act
        let result = mock_repo.find(message_id).await;

        // Assert
        assert!(result.is_err(), "Expected database error");
        match result.unwrap_err() {
            sqlx::Error::Configuration(_) => (), // ✅ Expected
            _ => panic!("Expected Configuration error"),
        }
    }
}
