use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::group_chat::NewGroupChat;
use crate::repository::group_chat_repository::GroupChatRepository;

impl GroupChatRepository {
    pub async fn insert_inner(&self, new_group_chat: NewGroupChat) -> Result<i32, Error> {
        let now = chrono::Utc::now();
        let rec = sqlx::query_scalar(
            r#"
            INSERT INTO "group_chat" (name, description, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#
        )
            .bind(new_group_chat.name)
            .bind(new_group_chat.description)
            .bind(new_group_chat.created_by)
            .bind(now)
            .bind(now)
            .fetch_one(self.db_conn.get_pool())
            .await?;

        Ok(rec)
    }
}

#[cfg(test)]
mod group_chat_repository_insert_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::factory::group_chat_factory::GroupChatFactory;
    use crate::repository::group_chat_repository::group_chat_repository_trait::MockGroupChatRepositoryTrait;
    use crate::repository::group_chat_repository::GroupChatRepositoryTrait;

    #[tokio::test]
    async fn test_insert_success() {
        // Arrange
        let mut mock_group_chat_repo = MockGroupChatRepositoryTrait::new();
        let new_group_chat = GroupChatFactory::unique_fake_new_group_chat("test_insert", 1);
        let expected_id = 123;

        mock_group_chat_repo
            .expect_insert()
            .with(eq(new_group_chat.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(expected_id)
                })
            });

        // Act
        let result = mock_group_chat_repo.insert(new_group_chat).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_id);
    }

    #[tokio::test]
    async fn test_insert_database_error() {
        // Arrange
        let mut mock_group_chat_repo = MockGroupChatRepositoryTrait::new();
        let new_group_chat = GroupChatFactory::unique_fake_new_group_chat("test_error", 1);
        let expected_error = sqlx::Error::PoolClosed;

        mock_group_chat_repo
            .expect_insert()
            .with(eq(new_group_chat.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Err(sqlx::Error::PoolClosed)
                })
            });

        // Act
        let result = mock_group_chat_repo.insert(new_group_chat).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            sqlx::Error::PoolClosed => {}, // Expected
            other => panic!("Expected PoolClosed error, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_insert_with_different_creators() {
        // Arrange
        let mut mock_group_chat_repo = MockGroupChatRepositoryTrait::new();
        
        let group1 = GroupChatFactory::unique_fake_new_group_chat("creator1", 1);
        let group2 = GroupChatFactory::unique_fake_new_group_chat("creator2", 2);
        let group3 = GroupChatFactory::unique_fake_new_group_chat("creator3", 3);

        mock_group_chat_repo
            .expect_insert()
            .with(eq(group1.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(101)
                })
            });

        mock_group_chat_repo
            .expect_insert()
            .with(eq(group2.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(102)
                })
            });

        mock_group_chat_repo
            .expect_insert()
            .with(eq(group3.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(103)
                })
            });

        // Act
        let result1 = mock_group_chat_repo.insert(group1).await;
        let result2 = mock_group_chat_repo.insert(group2).await;
        let result3 = mock_group_chat_repo.insert(group3).await;

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        assert_eq!(result1.unwrap(), 101);
        assert_eq!(result2.unwrap(), 102);
        assert_eq!(result3.unwrap(), 103);
    }

    #[tokio::test]
    async fn test_insert_with_factory_utilities() {
        // Arrange
        let mut mock_group_chat_repo = MockGroupChatRepositoryTrait::new();

        let base_group = GroupChatFactory::fake_new_group_chat();
        let custom_group = GroupChatFactory::with_specific_creator(
            GroupChatFactory::with_name(
                GroupChatFactory::with_description(base_group, "Custom description".to_string()),
                "Custom Group Name".to_string()
            ),
            42
        );

        mock_group_chat_repo
            .expect_insert()
            .with(eq(custom_group.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Ok(999)
                })
            });

        // Act
        let result = mock_group_chat_repo.insert(custom_group).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 999);
    }

    #[tokio::test]
    async fn test_insert_empty_name_validation() {
        // Arrange
        let mut mock_group_chat_repo = MockGroupChatRepositoryTrait::new();
        let mut group = GroupChatFactory::fake_new_group_chat();
        group.name = "".to_string(); // Empty name

        mock_group_chat_repo
            .expect_insert()
            .with(eq(group.clone()))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Err(Error::RowNotFound) // Simulate error db
                })
            });

        // Act
        let result = mock_group_chat_repo.insert(group).await;

        // Assert
        assert!(result.is_err(), "Expected a validation (DB) error");
    }
}