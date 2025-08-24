use crate::dto::text_message_dto::{TextMessageInfoReadDto, TextMessageLastSentAtDto};
use crate::entity::group_membership::MembershipStatus;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::text_message_error::TextMessageError;
use crate::service::text_message_service::TextMessageService;

impl TextMessageService {
    pub async fn find_info_last_sent_at_internal(&self, auth_user_id: i32, payload: TextMessageLastSentAtDto) -> Result<Option<TextMessageInfoReadDto>, ApiError> {
        // Check if he is an active member
        let membership = self.group_membership_service
            .find_active_by_user_id_and_group_id(auth_user_id, payload.group_chat_id)
            .await
            .map_err(|e| match e {
                ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                    ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)
                }
                _ => e,
            })?;
        if membership.membership_status != MembershipStatus::Active {
            return Err(ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages));
        }

        // Search the last sent at message for the user in the group
        let text_message = self.text_message_repo
            .find_info_last_sent(auth_user_id, payload.group_chat_id)
            .await
            .map_err(|e| ApiError::DbError(DbError::SomethingWentWrong(e.to_string())))?;

        match text_message {
            Some(ref text_message) => {
                // fai qualcosa con text_message
            },
            None => {
                // nessun record trovato
                println!("No last sent message found for user {}", auth_user_id);
            }
        }

        if text_message.is_none() {
            return Ok(None);
        }

        Ok(Some(TextMessageInfoReadDto::from(text_message.unwrap())))
    }
}

#[cfg(test)]
mod find_info_last_sent_at_service_tests {
    use super::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::dto::group_membership_dto::GroupMembershipReadDto;
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_find_info_last_sent_at_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        let auth_user_id = 1;
        let group_chat_id = 2;
        let payload = TextMessageLastSentAtDto { group_chat_id };
        
        let test_info = TextMessageFactory::fake_text_message_info_sent_only(1, auth_user_id, 3);
        let membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let membership_dto = GroupMembershipReadDto::from(membership);
        
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_chat_id))
            .times(1)
            .returning({
                let membership_dto = membership_dto.clone();
                move |_, _| Box::pin({
                    let membership_dto = membership_dto.clone();
                    async move { Ok(membership_dto) }
                })
            });

        mock_repo
            .expect_find_info_last_sent()
            .with(eq(auth_user_id), eq(group_chat_id))
            .times(1)
            .returning({
                let test_info = test_info.clone();
                move |_, _| Box::pin({
                    let test_info = test_info.clone();
                    async move { Ok(Some(test_info)) }
                })
            });

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_info_last_sent_at_internal(auth_user_id, payload).await;

        // Assert
        assert!(result.is_ok());
        let actual_dto = result.unwrap().unwrap();
        assert_eq!(actual_dto.id, test_info.id);
        assert_eq!(actual_dto.user_id, auth_user_id);
        assert_eq!(actual_dto.text_message_id, test_info.text_message_id);
        assert!(actual_dto.sent_at.is_some());
    }

    #[tokio::test]
    async fn test_find_info_last_sent_at_user_not_member() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        let auth_user_id = 1;
        let group_chat_id = 2;
        let payload = TextMessageLastSentAtDto { group_chat_id };
        
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_chat_id))
            .times(1)
            .returning(|_, _| Box::pin(async move { 
                Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound))
            }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_info_last_sent_at_internal(auth_user_id, payload).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
                // Expected error type
            }
            _ => panic!("Expected UserCannotAccessMessages error"),
        }
    }

    #[tokio::test]
    async fn test_find_info_last_sent_at_inactive_member() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        let auth_user_id = 1;
        let group_chat_id = 2;
        let payload = TextMessageLastSentAtDto { group_chat_id };
        
        let mut inactive_membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        inactive_membership.membership_status = MembershipStatus::Left;
        let membership_dto = GroupMembershipReadDto::from(inactive_membership);
        
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_chat_id))
            .times(1)
            .returning({
                let membership_dto = membership_dto.clone();
                move |_, _| Box::pin({
                    let membership_dto = membership_dto.clone();
                    async move { Ok(membership_dto) }
                })
            });

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_info_last_sent_at_internal(auth_user_id, payload).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages) => {
                // Expected error type
            }
            _ => panic!("Expected UserCannotAccessMessages error"),
        }
    }

    #[tokio::test]
    async fn test_find_info_last_sent_at_no_messages_found() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        let auth_user_id = 1;
        let group_chat_id = 2;
        let payload = TextMessageLastSentAtDto { group_chat_id };
        
        let membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let membership_dto = GroupMembershipReadDto::from(membership);
        
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_chat_id))
            .times(1)
            .returning({
                let membership_dto = membership_dto.clone();
                move |_, _| Box::pin({
                    let membership_dto = membership_dto.clone();
                    async move { Ok(membership_dto) }
                })
            });

        mock_repo
            .expect_find_info_last_sent()
            .with(eq(auth_user_id), eq(group_chat_id))
            .times(1)
            .returning(|_, _| Box::pin(async move { Ok(None) }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_info_last_sent_at_internal(auth_user_id, payload).await;

        // Assert
        assert!(result.is_ok(), "Error occurred: {:?}", result.as_ref().err());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_find_info_last_sent_at_database_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        let auth_user_id = 1;
        let group_chat_id = 2;
        let payload = TextMessageLastSentAtDto { group_chat_id };
        
        let membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let membership_dto = GroupMembershipReadDto::from(membership);
        
        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_chat_id))
            .times(1)
            .returning({
                let membership_dto = membership_dto.clone();
                move |_, _| Box::pin({
                    let membership_dto = membership_dto.clone();
                    async move { Ok(membership_dto) }
                })
            });

        mock_repo
            .expect_find_info_last_sent()
            .with(eq(auth_user_id), eq(group_chat_id))
            .times(1)
            .returning(|_, _| Box::pin(async move { 
                Err(sqlx::Error::Configuration("Database connection failed".into())) 
            }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_info_last_sent_at_internal(auth_user_id, payload).await;

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