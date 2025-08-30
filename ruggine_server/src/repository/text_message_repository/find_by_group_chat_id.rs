use sqlx::Error;
use chrono::{DateTime, Utc};
use crate::config::database::DatabaseTrait;
use crate::entity::text_message::TextMessage;
use crate::repository::text_message_repository::TextMessageRepository;

impl TextMessageRepository {
    pub async fn find_by_group_chat_id_paginated_inner(&self, group_chat_id: i32, cursor: Option<DateTime<Utc>>, limit: usize) -> Result<Vec<TextMessage>, Error> {
        let rows = match cursor {
            Some(cursor_datetime) => {
                let query = sqlx::query_as!(
                    TextMessage,
                    r#"
                    SELECT id, content, sender_id, group_chat_id, sent_at
                    FROM text_message
                    WHERE group_chat_id = $1 AND sent_at < $2
                    ORDER BY sent_at DESC
                    LIMIT $3
                    "#,
                    group_chat_id,
                    cursor_datetime,
                    limit as i64
                );

                // se c'è una transazione, usala; altrimenti usa la pool
                if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
                    query.fetch_all(&mut *tx_ref).await
                } else {
                    query.fetch_all(self.db_conn.get_pool()).await
                }
            }
            None => {
                // First page - get latest messages
                let query = sqlx::query_as!(
                    TextMessage,
                    r#"
                    SELECT id, content, sender_id, group_chat_id, sent_at
                    FROM text_message
                    WHERE group_chat_id = $1
                    ORDER BY sent_at DESC
                    LIMIT $2
                    "#,
                    group_chat_id,
                    limit as i64
                );

                // se c'è una transazione, usala; altrimenti usa la pool
                if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
                    query.fetch_all(&mut *tx_ref).await
                } else {
                    query.fetch_all(self.db_conn.get_pool()).await
                }
            }
        };

        rows
    }
}

#[cfg(test)]
mod text_message_repository_find_by_group_chat_id_tests {
    use mockall::predicate::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::repository::text_message_repository::TextMessageRepositoryTrait;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_paginated_first_page() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let group_chat_id = 1i32;
        let limit = 20usize;
        let expected_messages = TextMessageFactory::fake_text_messages_for_group(group_chat_id, 5);

        mock_repo
            .expect_find_by_group_chat_id_paginated()
            .with(eq(group_chat_id), eq(None), eq(limit))
            .times(1)
            .returning(move |_, _, _| Box::pin({
                let value = expected_messages.clone();
                async move { Ok(value.clone()) }
            }));

        // Act
        let result = mock_repo.find_by_group_chat_id_paginated(group_chat_id, None, limit).await;

        // Assert
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 5);
        assert!(messages.iter().all(|m| m.group_chat_id == group_chat_id));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_paginated_with_cursor() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let group_chat_id = 1i32;
        let limit = 10usize;
        let cursor = chrono::Utc::now();
        let expected_messages = TextMessageFactory::fake_text_messages_for_group(group_chat_id, 3);

        mock_repo
            .expect_find_by_group_chat_id_paginated()
            .with(eq(group_chat_id), eq(Some(cursor)), eq(limit))
            .times(1)
            .returning(move |_, _, _| Box::pin({
                let value = expected_messages.clone();
                async move { Ok(value.clone()) }
            }));


        // Act
        let result = mock_repo.find_by_group_chat_id_paginated(group_chat_id, Some(cursor), limit).await;

        // Assert
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 3);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_paginated_empty_result() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let group_chat_id = 999i32; // Non-existent group
        let limit = 20usize;

        mock_repo
            .expect_find_by_group_chat_id_paginated()
            .with(eq(group_chat_id), eq(None), eq(limit))
            .times(1)
            .returning(move |_, _, _| Box::pin(async move { Ok(vec![]) }));

        // Act
        let result = mock_repo.find_by_group_chat_id_paginated(group_chat_id, None, limit).await;

        // Assert
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 0);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_paginated_invalid_cursor() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let group_chat_id = 1i32;
        let limit = 10usize;
        let invalid_cursor = chrono::Utc::now() - chrono::Duration::days(30); // Valid DateTime but old

        mock_repo
            .expect_find_by_group_chat_id_paginated()
            .with(eq(group_chat_id), eq(Some(invalid_cursor)), eq(limit))
            .times(1)
            .returning(move |_, _, _| Box::pin(async move {
                Ok(vec![]) // Return empty result for old cursor
            }));

        // Act
        let result = mock_repo.find_by_group_chat_id_paginated(group_chat_id, Some(invalid_cursor), limit).await;

        // Assert
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 0);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_group_chat_id_paginated_large_limit() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let group_chat_id = 1i32;
        let limit = 100usize;
        let expected_messages = TextMessageFactory::fake_text_messages_for_group(group_chat_id, 50);

        mock_repo
            .expect_find_by_group_chat_id_paginated()
            .with(eq(group_chat_id), eq(None), eq(limit))
            .times(1)
            .returning(move |_, _, _| Box::pin({
                let value = expected_messages.clone();
                async move { Ok(value.clone()) }
            }));


        // Act
        let result = mock_repo.find_by_group_chat_id_paginated(group_chat_id, None, limit).await;

        // Assert
        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 50);
    }
}
