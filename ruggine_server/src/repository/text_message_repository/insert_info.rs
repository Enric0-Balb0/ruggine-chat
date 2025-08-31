use crate::{entity::text_message::NewTextMessageInfo, repository::text_message_repository::TextMessageRepository};
use sqlx::Error as SqlxError;
use tracing::info;
use crate::config::database::DatabaseTrait;

impl TextMessageRepository {
    pub async fn insert_text_message_info_inner(&self, text_message_info: NewTextMessageInfo) -> Result<i32, SqlxError> {
        let query = sqlx::query_scalar(
            r#"
            INSERT INTO text_message_info (user_id, text_message_id)
            VALUES ($1, $2)
            RETURNING id
            "#
        )
        .bind(text_message_info.user_id)
        .bind(text_message_info.text_message_id);

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
                eprintln!("Failed to insert text message info repository: {:?}", err);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod text_message_repository_insert_info_tests {
    use super::*;
    use crate::entity::text_message::NewTextMessageInfo;
    use mockall::predicate::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::repository::text_message_repository::TextMessageRepositoryTrait;
    use crate::utils::mock_database_error::MockDatabaseError;

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_text_message_info_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let text_message_info = TextMessageFactory::fake_new_text_message_info();
        let expected_id = 123i32;

        mock_repo
            .expect_insert_text_message_info()
            .with(function(|arg: &NewTextMessageInfo| {
                arg.user_id == 1 && arg.text_message_id == 1
            }))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(expected_id) }));

        // Act
        let result = mock_repo.insert_text_message_info(text_message_info).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_text_message_info_with_different_ids() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let text_message_info = TextMessageFactory::fake_new_text_message_info_with_ids(42, 100);
        let expected_id = 456i32;

        mock_repo
            .expect_insert_text_message_info()
            .with(function(|arg: &NewTextMessageInfo| {
                arg.user_id == 42 && arg.text_message_id == 100
            }))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(expected_id) }));

        // Act
        let result = mock_repo.insert_text_message_info(text_message_info).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_text_message_info_foreign_key_constraint_violation() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let text_message_info = TextMessageFactory::fake_new_text_message_info_with_ids(999, 999); // Non-existent IDs

        mock_repo
            .expect_insert_text_message_info()
            .with(function(|arg: &NewTextMessageInfo| {
                arg.user_id == 999 && arg.text_message_id == 999
            }))
            .times(1)
            .returning(|_| Box::pin(async {
                Err(MockDatabaseError::foreign_key_violation())
            }));

        // Act
        let result = mock_repo.insert_text_message_info(text_message_info).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_text_message_info_database_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let text_message_info = TextMessageFactory::fake_new_text_message_info();

        mock_repo
            .expect_insert_text_message_info()
            .with(function(|_: &NewTextMessageInfo| true))
            .times(1)
            .returning(|_| Box::pin(async move {
                Err(SqlxError::RowNotFound)
            }));

        // Act
        let result = mock_repo.insert_text_message_info(text_message_info).await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), SqlxError::RowNotFound);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_insert_text_message_info_with_factory_utility_methods() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let base_info = TextMessageFactory::fake_new_text_message_info();
        let text_message_info = TextMessageFactory::with_user_id_info(
            TextMessageFactory::with_text_message_id_info(base_info, 500),
            200
        );
        let expected_id = 789i32;

        mock_repo
            .expect_insert_text_message_info()
            .with(function(|arg: &NewTextMessageInfo| {
                arg.user_id == 200 && arg.text_message_id == 500
            }))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(expected_id) }));

        // Act
        let result = mock_repo.insert_text_message_info(text_message_info).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_id);
    }
}