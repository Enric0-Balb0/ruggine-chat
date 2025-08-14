use crate::dto::text_message_dto::TextMessageReadDto;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::text_message_error::TextMessageError;
use crate::service::text_message_service::TextMessageService;

impl TextMessageService {
    pub async fn find_by_id_internal(&self, id: i32) -> Result<TextMessageReadDto, ApiError> {
        let text_message = self.text_message_repo.find(id).await.map_err(|e| {
            let db_error = match e {
                sqlx::Error::RowNotFound => {
                    return ApiError::TextMessageError(TextMessageError::MessageNotFound);
                }
                sqlx::Error::Database(db_err) => {
                    if let Some(code) = db_err.code() {
                        if code == "23503" {
                            return ApiError::DbError(DbError::ForeignKeyViolation(db_err.to_string()));
                        }
                    }
                    ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string()))
                }
                _ => ApiError::DbError(DbError::SomethingWentWrong(e.to_string())),
            };
            db_error
        })?;

        Ok(TextMessageReadDto::from(text_message))
    }
}

#[cfg(test)]
mod find_by_id_service_tests {
    use super::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::service::text_message_service::text_message_service_trait::TextMessageServiceTrait;
    use mockall::predicate::*;
    use std::sync::Arc;
    use crate::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;

    #[tokio::test]
    async fn test_find_by_id_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let test_message = TextMessageFactory::fake_text_message_with_id(1);
        let expected_dto = TextMessageReadDto::from(test_message.clone());
        
        mock_repo
            .expect_find()
            .with(eq(1))
            .times(1)
            .returning({
                let test_message = test_message.clone();
                move |_| Box::pin({
                    let test_message = test_message.clone();
                    async move { Ok(test_message) }
                })
            });

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(MockGroupChatServiceTrait::new()));

        // Act
        let result = service.find_by_id_internal(1).await;

        // Assert
        assert!(result.is_ok());
        let actual_dto = result.unwrap();
        assert_eq!(actual_dto.id, expected_dto.id);
        assert_eq!(actual_dto.content, expected_dto.content);
        assert_eq!(actual_dto.sender_id, expected_dto.sender_id);
        assert_eq!(actual_dto.group_chat_id, expected_dto.group_chat_id);
    }

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        
        mock_repo
            .expect_find()
            .with(eq(999))
            .times(1)
            .returning(|_| Box::pin(async move { Err(sqlx::Error::RowNotFound) }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(MockGroupChatServiceTrait::new()));

        // Act
        let result = service.find_by_id_internal(999).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::TextMessageError(TextMessageError::MessageNotFound) => {
                // Expected error type
            }
            _ => panic!("Expected MessageNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_find_by_id_database_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mock_group_member_membership_service = MockGroupMembershipServiceTrait::new();

        mock_repo
            .expect_find()
            .with(eq(1))
            .times(1)
            .returning(|_| Box::pin(async move { 
                Err(sqlx::Error::Configuration("Database connection failed".into())) 
            }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_member_membership_service), Arc::new(MockGroupChatServiceTrait::new()));

        // Act
        let result = service.find_by_id_internal(1).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(_)) => {
                // Expected error type
            }
            _ => panic!("Expected SomethingWentWrong error"),
        }
    }
}
