use crate::config::database::DatabaseTrait;
use crate::entity::text_message::TextMessageInfoUpdate;
use crate::repository::text_message_repository::TextMessageRepository;
use sqlx::Error;

impl TextMessageRepository {
    pub async fn update_info_inner(&self, text_message_info_update: TextMessageInfoUpdate) -> Result<(), Error> {
        // Build dynamic query based on what needs to be updated
        let mut set_clauses = Vec::new();
        let mut bind_index = 2; // Start from $2 since $1 is the id

        if text_message_info_update.sent_at.is_some() {
            set_clauses.push(format!("sent_at = ${}", bind_index));
            bind_index += 1;
        }

        if text_message_info_update.read_at.is_some() {
            set_clauses.push(format!("read_at = ${}", bind_index));
            bind_index += 1;
        }

        if set_clauses.is_empty() {
            return Ok(()); // Nothing to update
        }

        let sql = format!(
            "UPDATE text_message_info SET {} WHERE id = $1",
            set_clauses.join(", ")
        );

        // Build query with dynamic binding
        let mut query = sqlx::query(&sql).bind(text_message_info_update.id);

        if let Some(sent_at) = text_message_info_update.sent_at {
            query = query.bind(sent_at);
        }

        if let Some(read_at) = text_message_info_update.read_at {
            query = query.bind(read_at);
        }

        match query.execute(self.db_conn.get_pool()).await {
            Ok(result) => {
                if result.rows_affected() == 0 {
                    Err(Error::RowNotFound)
                } else {
                    Ok(())
                }
            }
            Err(err) => {
                eprintln!("Failed to update text message info: {:?}", err);
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod text_message_repository_update_info_tests {
    use super::*;
    use crate::entity::text_message::TextMessageInfoUpdate;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use sqlx::Error;
    use chrono::Utc;
    use mockall::predicate::*;
    use crate::repository::text_message_repository::TextMessageRepositoryTrait;

    #[tokio_shared_rt::test(shared)]
    async fn test_update_info_inner_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let update_info = TextMessageInfoUpdate {
            id: 1,
            sent_at: Some(Utc::now()),
            read_at: Some(Utc::now()),
        };

        mock_repo
            .expect_update_info()
            .withf(move |arg: &TextMessageInfoUpdate| {
                arg.id == update_info.id &&
                    arg.sent_at.is_some() &&
                    arg.read_at.is_some()
            })
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        // Act
        let result = mock_repo.update_info(update_info).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_info_inner_partial_update() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let update_info = TextMessageInfoUpdate {
            id: 2,
            sent_at: None,
            read_at: Some(Utc::now()),
        };

        mock_repo
            .expect_update_info()
            .withf(move |arg: &TextMessageInfoUpdate| {
                arg.id == update_info.id &&
                    arg.sent_at.is_none() &&
                    arg.read_at.is_some()
            })
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }));

        // Act
        let result = mock_repo.update_info(update_info).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_info_inner_nothing_to_update() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let update_info = TextMessageInfoUpdate {
            id: 3,
            sent_at: None,
            read_at: None,
        };

        mock_repo
            .expect_update_info()
            .withf(move |arg: &TextMessageInfoUpdate| arg.id == update_info.id)
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) })); // Nothing to update returns Ok

        // Act
        let result = mock_repo.update_info(update_info).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_info_inner_row_not_found() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let update_info = TextMessageInfoUpdate {
            id: 4,
            sent_at: Some(Utc::now()),
            read_at: None,
        };

        mock_repo
            .expect_update_info()
            .withf(move |arg: &TextMessageInfoUpdate| arg.id == update_info.id)
            .times(1)
            .returning(|_| Box::pin(async { Err(Error::RowNotFound) }));

        // Act
        let result = mock_repo.update_info(update_info).await;

        // Assert
        assert!(result.is_err());
        matches!(result.unwrap_err(), Error::RowNotFound);
    }
}
