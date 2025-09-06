use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::text_message::TextMessageInfo;
use crate::repository::text_message_repository::TextMessageRepository;

impl TextMessageRepository {
    pub async fn find_info_by_id_inner(&self, id: i32) -> Result<TextMessageInfo, Error> {
        let query = sqlx::query_as::<_, TextMessageInfo>(
            r#"
            SELECT 
                id, 
                user_id, 
                text_message_id, 
                sent_at, 
                read_at
            FROM text_message_info
            WHERE id = $1
            "#
        )
        .bind(id);


        // se c'è una transazione, usala; altrimenti usa la pool
        if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            query.fetch_one(&mut *tx_ref).await
        } else {
            query.fetch_one(self.db_conn.get_pool()).await
        }
    }
}

#[cfg(test)]
mod text_message_repository_find_info_by_id_tests {
    use mockall::predicate::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::repository::text_message_repository::TextMessageRepositoryTrait;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_info_by_id_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let info_id = 123i32;
        let user_id = 456i32;
        let text_message_id = 789i32;
        let expected_info = TextMessageFactory::fake_text_message_info_with_ids(
            info_id, 
            user_id, 
            text_message_id
        );

        mock_repo
            .expect_find_info_by_id()
            .with(eq(info_id))
            .times(1)
            .returning(move |_| Box::pin({
                let value = expected_info.clone();
                async move { Ok(value.clone()) }
            }));

        // Act
        let result = mock_repo.find_info_by_id(info_id).await;

        // Assert
        assert!(result.is_ok(), "Failed to find text message info");
        let info = result.unwrap();
        assert_eq!(info.id, info_id);
        assert_eq!(info.user_id, user_id);
        assert_eq!(info.text_message_id, text_message_id);
        assert!(info.sent_at.is_some());
        assert!(info.read_at.is_none());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_info_by_id_with_read_timestamp() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let info_id = 321i32;
        let user_id = 654i32;
        let text_message_id = 987i32;
        let expected_info = TextMessageFactory::fake_text_message_info_read_dto_with_ids_and_read_at(
            info_id, 
            user_id, 
            text_message_id
        );
        let expected_info_entity = TextMessageFactory::fake_text_message_info_with_ids(
            info_id,
            user_id,
            text_message_id
        );

        mock_repo
            .expect_find_info_by_id()
            .with(eq(info_id))
            .times(1)
            .returning(move |_| Box::pin({
                let value = expected_info_entity.clone();
                async move { Ok(value.clone()) }
            }));

        // Act
        let result = mock_repo.find_info_by_id(info_id).await;

        // Assert
        assert!(result.is_ok(), "Failed to find text message info");
        let info = result.unwrap();
        assert_eq!(info.id, info_id);
        assert_eq!(info.user_id, user_id);
        assert_eq!(info.text_message_id, text_message_id);
        assert!(info.sent_at.is_some());
        assert!(info.read_at.is_none());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_info_by_id_not_found() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let non_existent_id = 999999i32;

        mock_repo
            .expect_find_info_by_id()
            .with(eq(non_existent_id))
            .times(1)
            .returning(|_| Box::pin(async move {
                Err(sqlx::Error::RowNotFound)
            }));

        // Act
        let result = mock_repo.find_info_by_id(non_existent_id).await;

        // Assert
        assert!(result.is_err(), "Expected row not found error");
        matches!(result.unwrap_err(), sqlx::Error::RowNotFound);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_info_by_id_database_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let info_id = 123i32;

        mock_repo
            .expect_find_info_by_id()
            .with(eq(info_id))
            .times(1)
            .returning(|_| Box::pin(async move {
                Err(sqlx::Error::Configuration("Database connection failed".into()))
            }));

        // Act
        let result = mock_repo.find_info_by_id(info_id).await;

        // Assert
        assert!(result.is_err(), "Expected database error");
        match result.unwrap_err() {
            sqlx::Error::Configuration(_) => (), // Expected
            _ => panic!("Expected Configuration error"),
        }
    }
}