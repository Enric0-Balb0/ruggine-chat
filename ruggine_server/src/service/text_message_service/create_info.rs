use crate::dto::text_message_dto::{TextMessageInfoCreateDto, TextMessageInfoReadDto};
use crate::entity::text_message::NewTextMessageInfo;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::text_message_error::TextMessageError;
use crate::service::text_message_service::{TextMessageService, TextMessageServiceTrait};

impl TextMessageService {
    pub async fn create_info_internal(
        &self,
        payload: TextMessageInfoCreateDto,
        sender_id: i32,
    ) -> Result<TextMessageInfoReadDto, ApiError> {
        // Check if the message exists first
        let message = self
            .find_by_id(payload.text_message_id)
            .await
            .map_err(|e| match e {
                ApiError::TextMessageError(TextMessageError::MessageNotFound) => {
                    ApiError::TextMessageError(TextMessageError::MessageNotFound)
                }
                _ => e,
            })?;

        // Check if the sender is the creator of the message
        if message.sender_id != sender_id {
            return Err(ApiError::TextMessageError(TextMessageError::UserIsNotCreator));
        }

        // Create the new text message entity
        let new_text_message = NewTextMessageInfo {
            user_id: payload.user_id,
            text_message_id: message.id,
        };

        // Insert the message into the database
        let message_info_id = self
            .text_message_repo
            .insert_text_message_info(new_text_message)
            .await
            .map_err(|e| match e {
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        "23503" => ApiError::DbError(DbError::ForeignKeyViolation(db_err.to_string())),
                        "P0001" => {
                            // Trigger failure
                            if db_err.message().contains("NULL") {
                                ApiError::TextMessageError(TextMessageError::CannotSetReadAtBeforeSentAt)
                            } else {
                                ApiError::TextMessageError(TextMessageError::ReadAtMustBeGreaterOrEqualsToSentAt)
                            }
                        }
                        _ => ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string())),
                    }
                } else {
                    ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string()))
                }
            }
            _ => ApiError::DbError(DbError::SomethingWentWrong(e.to_string())),
            })?;


        // Retrieve the created message to return it
        let message_info = self.text_message_repo
            .find_info_by_id(message_info_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
                    ApiError::TextMessageError(TextMessageError::MessageNotFound)
                }
                _ => ApiError::DbError(DbError::SomethingWentWrong(e.to_string())),
            })?;

        Ok(TextMessageInfoReadDto::from(message_info))
    }
}
#[cfg(test)]
mod create_info_service_tests {
    use super::*;
    use crate::error::{api_error::ApiError, db_error::DbError, text_message_error::TextMessageError};
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;
    use crate::utils::mock_database_error::MockDatabaseError;
    use mockall::predicate::*;
    use std::sync::Arc;

    fn create_service_with_mock_repo(mock_repo: MockTextMessageRepositoryTrait) -> TextMessageService {
        TextMessageService {
            text_message_repo: Arc::new(mock_repo),
            group_membership_service: Arc::new(MockGroupMembershipServiceTrait::new()),
            group_chat_service: Arc::new(MockGroupChatServiceTrait::new())
        }
    }

    #[tokio::test]
    async fn test_create_info_internal_success() {
        // Arrange
        let sender_id = 1;
        let text_message = TextMessageFactory::fake_text_message_with_ids(1, sender_id, 1);
        let message_info = TextMessageFactory::fake_text_message_info_read_dto_with_ids_and_read_at(1, sender_id, text_message.id);
        let message_info_entity = TextMessageFactory::fake_text_message_info_with_ids(1, sender_id, text_message.id);
        let payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(sender_id, text_message.id);

        let text_message_clone = text_message.clone();
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        mock_repo.expect_find()
            .with(eq(text_message.id))
            .times(1)
            .returning(move |_| {
                let tm = text_message_clone.clone();
                Box::pin(async move { Ok(tm) })
            });
        mock_repo.expect_insert_text_message_info()
            .with(always())
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(message_info.id) }));
        mock_repo.expect_find_info_by_id()
            .with(eq(message_info.id))
            .times(1)
            .returning(move |_| {
                let info = message_info_entity.clone();
                Box::pin(async move { Ok(info) })
            });

        let service = create_service_with_mock_repo(mock_repo);

        // Act
        let result = service.create_info_internal(payload.clone(), sender_id).await;

        // Assert
        assert!(result.is_ok());
        let info_read_dto = result.unwrap();
        assert_eq!(info_read_dto.id, message_info.id);
        assert_eq!(info_read_dto.user_id, sender_id);
        assert_eq!(info_read_dto.text_message_id, text_message.id);
    }

    #[tokio::test]
    async fn test_create_info_internal_message_not_found() {
        let sender_id = 1;
        let payload = TextMessageFactory::fake_text_message_info_create_dto();

        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        mock_repo.expect_find()
            .with(always())
            .times(1)
            .returning(|_| Box::pin(async move {
                Err(sqlx::Error::RowNotFound)
            }));

        let service = create_service_with_mock_repo(mock_repo);

        let result = service.create_info_internal(payload, sender_id).await;
        assert!(matches!(result.unwrap_err(), ApiError::TextMessageError(TextMessageError::MessageNotFound)));
    }

    #[tokio::test]
    async fn test_create_info_internal_user_not_creator() {
        let sender_id = 999; // Non-creator
        let text_message = TextMessageFactory::fake_text_message_with_ids(1, 1, 1);
        let payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(sender_id, text_message.id);

        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        mock_repo.expect_find()
            .with(eq(text_message.id))
            .times(1)
            .returning(move |_| {
                let tm = text_message.clone();
                Box::pin(async move { Ok(tm) })
            });

        let service = create_service_with_mock_repo(mock_repo);

        let result = service.create_info_internal(payload, sender_id).await;
        assert!(matches!(result.unwrap_err(), ApiError::TextMessageError(TextMessageError::UserIsNotCreator)));
    }

    #[tokio::test]
    async fn test_create_info_internal_insert_failure_foreign_key() {
        let sender_id = 1;
        let text_message = TextMessageFactory::fake_text_message_with_ids(1, sender_id, 1);
        let payload = TextMessageFactory::fake_text_message_info_create_dto_with_ids(sender_id, text_message.id);

        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        mock_repo.expect_find()
            .with(eq(text_message.id))
            .returning(move |_| {
                let tm = text_message.clone();
                Box::pin(async move { Ok(tm) })
            });
        mock_repo.expect_insert_text_message_info()
            .with(always())
            .returning(|_| Box::pin(async move {
                Err(MockDatabaseError::foreign_key_violation())
            }));

        let service = create_service_with_mock_repo(mock_repo);

        let result = service.create_info_internal(payload, sender_id).await;
        assert!(matches!(result.unwrap_err(), ApiError::DbError(DbError::ForeignKeyViolation(_))));
    }
}
