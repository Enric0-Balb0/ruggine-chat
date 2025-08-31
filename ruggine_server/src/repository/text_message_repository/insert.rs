use sqlx::Error;
use tracing::info;
use crate::config::database::DatabaseTrait;
use crate::entity::text_message::NewTextMessage;
use crate::repository::text_message_repository::TextMessageRepository;

impl TextMessageRepository {
    pub async fn insert_inner(&self, new_text_message: NewTextMessage) -> Result<i32, Error> {
        let now = chrono::Utc::now();
        let query = sqlx::query_scalar(
            r#"
            INSERT INTO text_message (content, sender_id, group_chat_id, sent_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#
        )
        .bind(new_text_message.content)
        .bind(new_text_message.sender_id)
        .bind(new_text_message.group_chat_id)
        .bind(now);

        // se c'è una transazione, usala; altrimenti usa la pool
        let response;

        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            info!("Using transaction");
            response = query.fetch_one(&mut *tx_ref).await
        } else {
            info!("Using pool");
            response = query.fetch_one(self.db_conn.get_pool()).await
        }

        match response {
            Ok(id) => Ok(id),
            Err(err) => {
                eprintln!("Failed to insert text message repository: {:?}", err);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod text_message_repository_insert_tests {
    use super::*;
    use crate::entity::text_message::NewTextMessage;
    use mockall::predicate::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::repository::text_message_repository::TextMessageRepositoryTrait;
    use crate::utils::mock_database_error::MockDatabaseError;

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let new_message = TextMessageFactory::fake_new_text_message();
        let expected_id = 123i32;

        mock_repo
            .expect_insert()
            .with(function(|arg: &NewTextMessage| {
                arg.content == "Test message content" &&
                arg.sender_id == 1 &&
                arg.group_chat_id == 1
            }))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(expected_id) }));

        // Act
        let result = mock_repo.insert(new_message).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_with_different_ids() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let new_message = TextMessageFactory::fake_new_text_message_with_ids(42, 10);
        let expected_id = 456i32;

        mock_repo
            .expect_insert()
            .with(function(|arg: &NewTextMessage| {
                arg.sender_id == 42 && arg.group_chat_id == 10
            }))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(expected_id) }));

        // Act
        let result = mock_repo.insert(new_message).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_failure_database_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let new_message = TextMessageFactory::fake_new_text_message();

        mock_repo
            .expect_insert()
            .with(function(|_: &NewTextMessage| true))
            .times(1)
            .returning(|_| Box::pin(async move {
                Err(sqlx::Error::RowNotFound)
            }));

        // Act
        let result = mock_repo.insert(new_message).await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), sqlx::Error::RowNotFound);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_foreign_key_constraint_violation() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let new_message = TextMessageFactory::fake_new_text_message_with_ids(999, 999); // Non-existent IDs

        mock_repo
            .expect_insert()
            .with(function(|arg: &NewTextMessage| {
                arg.sender_id == 999 && arg.group_chat_id == 999
            }))
            .times(1)
            .returning(|_| Box::pin(async {
                Err(MockDatabaseError::constraint_violation())
            }));

        // Act
        let result = mock_repo.insert(new_message).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_with_long_content() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let long_content = "a".repeat(10000);
        let new_message = TextMessageFactory::fake_new_text_message_with_content(long_content.clone());
        let expected_id = 789i32;

        mock_repo
            .expect_insert()
            .with(function(move |arg: &NewTextMessage| {
                arg.content.len() == 10000
            }))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(expected_id) }));

        // Act
        let result = mock_repo.insert(new_message).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_id);
    }
}
