use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::group_chat::GroupChat;
use crate::repository::group_chat_repository::GroupChatRepository;

impl GroupChatRepository {
    pub async fn find_by_id_inner(&self, id: i32) -> Result<GroupChat, Error> {
        let query = sqlx::query_as::<_, GroupChat>("SELECT * FROM \"group_chat\" WHERE id = $1")
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
mod group_chat_repository_find_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::repository::group_chat_repository::group_chat_repository_trait::MockGroupChatRepositoryTrait;
    use crate::repository::group_chat_repository::GroupChatRepositoryTrait;
    use crate::factory::group_chat_factory::GroupChatFactory;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_success() {
        // Arrange
        let mut mock_group_chat_repo = MockGroupChatRepositoryTrait::new();
        let expected_group = GroupChatFactory::fake_group_chat();
        let group_id = expected_group.id;

        mock_group_chat_repo
            .expect_find_by_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let group = GroupChatFactory::fake_group_chat();
                Box::pin(async move { Ok(group) })
            });

        // Act
        let result = mock_group_chat_repo.find_by_id(group_id).await;

        // Assert
        assert!(result.is_ok());
        let found_group = result.unwrap();
        assert_eq!(found_group.id, expected_group.id);
        assert_eq!(found_group.name, expected_group.name);
        assert_eq!(found_group.description, expected_group.description);
        assert_eq!(found_group.created_by, expected_group.created_by);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_not_found() {
        // Arrange
        let mut mock_group_chat_repo = MockGroupChatRepositoryTrait::new();
        let non_existent_id = 999;

        mock_group_chat_repo
            .expect_find_by_id()
            .with(eq(non_existent_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Err(Error::RowNotFound)
                })
            });

        // Act
        let result = mock_group_chat_repo.find_by_id(non_existent_id).await;

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_database_error() {
        // Arrange
        let mut mock_group_chat_repo = MockGroupChatRepositoryTrait::new();
        let group_id = 1;

        mock_group_chat_repo
            .expect_find_by_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Err(Error::PoolClosed)
                })
            });

        // Act
        let result = mock_group_chat_repo.find_by_id(group_id).await;

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PoolClosed));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_with_different_groups() {
        // Arrange
        let mut mock_group_chat_repo = MockGroupChatRepositoryTrait::new();
        
        let group1 = GroupChatFactory::fake_group_chat();
        let mut group2 = GroupChatFactory::fake_group_chat();
        group2.id = 2;
        group2.name = "Different Group".to_string();

        let group1_clone = group1.clone();
        let group2_clone = group2.clone();

        mock_group_chat_repo
            .expect_find_by_id()
            .with(eq(group1.id))
            .times(1)
            .returning(move |_| {
                let group = group1_clone.clone();
                Box::pin(async move { Ok(group) })
            });

        mock_group_chat_repo
            .expect_find_by_id()
            .with(eq(group2.id))
            .times(1)
            .returning(move |_| {
                let group = group2_clone.clone();
                Box::pin(async move { Ok(group) })
            });

        // Act
        let result1 = mock_group_chat_repo.find_by_id(group1.id).await;
        let result2 = mock_group_chat_repo.find_by_id(group2.id).await;

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        let found_group1 = result1.unwrap();
        let found_group2 = result2.unwrap();
        
        assert_eq!(found_group1.id, group1.id);
        assert_eq!(found_group2.id, group2.id);
        assert_ne!(found_group1.name, found_group2.name);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_invalid_id() {
        // Arrange
        let mut mock_group_chat_repo = MockGroupChatRepositoryTrait::new();
        let invalid_id = -1;

        mock_group_chat_repo
            .expect_find_by_id()
            .with(eq(invalid_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move {
                    Err(Error::RowNotFound)
                })
            });

        // Act
        let result = mock_group_chat_repo.find_by_id(invalid_id).await;

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound));
    }
}