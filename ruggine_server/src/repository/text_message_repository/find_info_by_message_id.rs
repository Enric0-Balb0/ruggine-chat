use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::text_message::TextMessageInfo;
use crate::repository::text_message_repository::TextMessageRepository;

impl TextMessageRepository {
    pub async fn find_info_by_message_id_inner(&self, message_id: i32) -> Result<Vec<TextMessageInfo>, Error> {
        let rec = sqlx::query_as::<_, TextMessageInfo>(
            r#"
            SELECT id, 
                user_id, 
                text_message_id, 
                sent_at, 
                read_at
            FROM text_message_info
            WHERE text_message_id = $1
            "#
        )
        .bind(message_id)
        .fetch_all(self.db_conn.get_pool())
        .await?;

        Ok(rec)
    }
}

#[cfg(test)]
mod text_message_repository_find_info_by_message_id_tests {
    use mockall::predicate::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::repository::text_message_repository::TextMessageRepositoryTrait;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_info_by_message_id_success_multiple() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let message_id = 789i32;
        let info1 = TextMessageFactory::fake_text_message_info_with_ids(1, 100, message_id);
        let info2 = TextMessageFactory::fake_text_message_info_with_ids(2, 101, message_id);
        let expected_list = vec![info1.clone(), info2.clone()];

        mock_repo
            .expect_find_info_by_message_id()
            .with(eq(message_id))
            .times(1)
            .returning(move |_| Box::pin({
                let values = expected_list.clone();
                async move { Ok(values.clone()) }
            }));

        // Act
        let result = mock_repo.find_info_by_message_id(message_id).await;

        // Assert
        assert!(result.is_ok(), "Failed to fetch infos by message id");
        let infos = result.unwrap();
        assert_eq!(infos.len(), 2);
        assert_eq!(infos[0].text_message_id, message_id);
        assert_eq!(infos[1].text_message_id, message_id);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_info_by_message_id_empty_list() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let message_id = 12345i32;

        mock_repo
            .expect_find_info_by_message_id()
            .with(eq(message_id))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(vec![]) }));

        // Act
        let result = mock_repo.find_info_by_message_id(message_id).await;

        // Assert
        assert!(result.is_ok(), "Expected empty result but got error");
        let infos = result.unwrap();
        assert!(infos.is_empty(), "Expected empty list for message_id with no results");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_info_by_message_id_database_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let message_id = 777i32;

        mock_repo
            .expect_find_info_by_message_id()
            .with(eq(message_id))
            .times(1)
            .returning(|_| Box::pin(async move {
                Err(sqlx::Error::Configuration("Database error".into()))
            }));

        // Act
        let result = mock_repo.find_info_by_message_id(message_id).await;

        // Assert
        assert!(result.is_err(), "Expected database error");
        match result.unwrap_err() {
            sqlx::Error::Configuration(_) => (), // ok
            _ => panic!("Expected Configuration error"),
        }
    }
}
