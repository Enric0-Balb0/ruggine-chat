use crate::dto::text_message_dto::{TextMessageInfoReadDto, TextMessageReadDto};
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::text_message_error::TextMessageError;
use crate::service::text_message_service::TextMessageService;

impl TextMessageService {
    pub async fn find_info_by_id_internal(&self, id: i32) -> Result<TextMessageInfoReadDto, ApiError> {
        let text_message_info = self.text_message_repo.find_info_by_id(id).await.map_err(|e| {
            let db_error = match e {
                sqlx::Error::RowNotFound => {
                    return ApiError::TextMessageError(TextMessageError::MessageInfoNotFound);
                }
                _ => ApiError::DbError(DbError::SomethingWentWrong(e.to_string())),
            };
            db_error
        })?;

        Ok(TextMessageInfoReadDto::from(text_message_info))
    }
}

#[cfg(test)]
mod find_info_by_id_service_tests {
    use super::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::service::text_message_service::text_message_service_trait::TextMessageServiceTrait;
    use crate::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_find_info_by_id_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        let test_info = TextMessageFactory::fake_text_message_info_with_ids(1, 2, 3);
        let expected_dto = TextMessageInfoReadDto::from(test_info.clone());
        
        mock_repo
            .expect_find_info_by_id()
            .with(eq(1))
            .times(1)
            .returning({
                let test_info = test_info.clone();
                move |_| Box::pin({
                    let test_info = test_info.clone();
                    async move { Ok(test_info) }
                })
            });

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_info_by_id_internal(1).await;

        // Assert
        assert!(result.is_ok());
        let actual_dto = result.unwrap();
        assert_eq!(actual_dto.id, expected_dto.id);
        assert_eq!(actual_dto.user_id, expected_dto.user_id);
        assert_eq!(actual_dto.text_message_id, expected_dto.text_message_id);
        assert_eq!(actual_dto.sent_at, expected_dto.sent_at);
        assert_eq!(actual_dto.read_at, expected_dto.read_at);
    }

    #[tokio::test]
    async fn test_find_info_by_id_not_found() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        mock_repo
            .expect_find_info_by_id()
            .with(eq(999))
            .times(1)
            .returning(|_| Box::pin(async move { Err(sqlx::Error::RowNotFound) }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_info_by_id_internal(999).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::TextMessageError(TextMessageError::MessageInfoNotFound) => {
                // Expected error type
            }
            _ => panic!("Expected MessageInfoNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_find_info_by_id_database_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();

        mock_repo
            .expect_find_info_by_id()
            .with(eq(1))
            .times(1)
            .returning(|_| Box::pin(async move { 
                Err(sqlx::Error::Configuration("Database connection failed".into())) 
            }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_info_by_id_internal(1).await;

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