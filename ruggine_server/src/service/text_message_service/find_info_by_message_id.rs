use crate::dto::text_message_dto::TextMessageInfoReadDto;
use crate::entity::group_membership::MembershipStatus;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::text_message_error::TextMessageError;
use crate::service::text_message_service::{TextMessageService, TextMessageServiceTrait};

impl TextMessageService {
    pub async fn find_info_by_message_id_internal(
        &self,
        auth_user_id: i32,
        message_id: i32,
    ) -> Result<Vec<TextMessageInfoReadDto>, ApiError> {
        // Check if the user is an active member of the group
        let text_message = self.find_by_id(message_id).await?;
        let membership = self.group_membership_service
            .find_active_by_user_id_and_group_id(auth_user_id, text_message.group_chat_id)
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

        // Return text message info
        let infos = self.text_message_repo
            .find_info_by_message_id(message_id)
            .await
            .map_err(|e| ApiError::DbError(DbError::SomethingWentWrong(e.to_string())))?;

        Ok(infos.into_iter().map(TextMessageInfoReadDto::from).collect())
    }
}

#[cfg(test)]
mod find_info_by_message_id_service_tests {
    use super::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::service::text_message_service::text_message_service_trait::TextMessageServiceTrait;
    use crate::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::dto::group_membership_dto::GroupMembershipReadDto;
    use mockall::predicate::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_find_info_by_message_id_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        let auth_user_id = 1;
        let message_id = 2;
        let group_chat_id = 3;
        
        let test_message = TextMessageFactory::fake_text_message_with_ids(message_id, auth_user_id, group_chat_id);
        let test_infos = TextMessageFactory::fake_text_message_infos_for_message(message_id, 3);
        let membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let membership_dto = GroupMembershipReadDto::from(membership);
        
        mock_repo
            .expect_find()
            .with(eq(message_id))
            .times(1)
            .returning({
                let test_message = test_message.clone();
                move |_| Box::pin({
                    let test_message = test_message.clone();
                    async move { Ok(test_message) }
                })
            });

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
            .expect_find_info_by_message_id()
            .with(eq(message_id))
            .times(1)
            .returning({
                let test_infos = test_infos.clone();
                move |_| Box::pin({
                    let test_infos = test_infos.clone();
                    async move { Ok(test_infos) }
                })
            });

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_info_by_message_id_internal(auth_user_id, message_id).await;

        // Assert
        assert!(result.is_ok());
        let actual_dtos = result.unwrap();
        assert_eq!(actual_dtos.len(), 3);
        assert_eq!(actual_dtos[0].text_message_id, message_id);
        assert_eq!(actual_dtos[1].text_message_id, message_id);
        assert_eq!(actual_dtos[2].text_message_id, message_id);
    }

    #[tokio::test]
    async fn test_find_info_by_message_id_empty_result() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        let auth_user_id = 1;
        let message_id = 2;
        let group_chat_id = 3;
        
        let test_message = TextMessageFactory::fake_text_message_with_ids(message_id, auth_user_id, group_chat_id);
        let membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let membership_dto = GroupMembershipReadDto::from(membership);
        
        mock_repo
            .expect_find()
            .with(eq(message_id))
            .times(1)
            .returning({
                let test_message = test_message.clone();
                move |_| Box::pin({
                    let test_message = test_message.clone();
                    async move { Ok(test_message) }
                })
            });

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
            .expect_find_info_by_message_id()
            .with(eq(message_id))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(vec![]) }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_info_by_message_id_internal(auth_user_id, message_id).await;

        // Assert
        assert!(result.is_ok());
        let actual_dtos = result.unwrap();
        assert_eq!(actual_dtos.len(), 0);
    }

    #[tokio::test]
    async fn test_find_info_by_message_id_message_not_found() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        let auth_user_id = 1;
        let message_id = 999;
        
        mock_repo
            .expect_find()
            .with(eq(message_id))
            .times(1)
            .returning(|_| Box::pin(async move { Err(sqlx::Error::RowNotFound) }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_info_by_message_id_internal(auth_user_id, message_id).await;

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
    async fn test_find_info_by_message_id_user_not_member() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        let auth_user_id = 1;
        let message_id = 2;
        let group_chat_id = 3;
        
        let test_message = TextMessageFactory::fake_text_message_with_ids(message_id, auth_user_id, group_chat_id);
        
        mock_repo
            .expect_find()
            .with(eq(message_id))
            .times(1)
            .returning({
                let test_message = test_message.clone();
                move |_| Box::pin({
                    let test_message = test_message.clone();
                    async move { Ok(test_message) }
                })
            });

        mock_group_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_chat_id))
            .times(1)
            .returning(|_, _| Box::pin(async move { 
                Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound))
            }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_info_by_message_id_internal(auth_user_id, message_id).await;

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
    async fn test_find_info_by_message_id_inactive_member() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        let auth_user_id = 1;
        let message_id = 2;
        let group_chat_id = 3;
        
        let test_message = TextMessageFactory::fake_text_message_with_ids(message_id, auth_user_id, group_chat_id);
        let mut inactive_membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        inactive_membership.membership_status = MembershipStatus::Left;
        let membership_dto = GroupMembershipReadDto::from(inactive_membership);
        
        mock_repo
            .expect_find()
            .with(eq(message_id))
            .times(1)
            .returning({
                let test_message = test_message.clone();
                move |_| Box::pin({
                    let test_message = test_message.clone();
                    async move { Ok(test_message) }
                })
            });

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
        let result = service.find_info_by_message_id_internal(auth_user_id, message_id).await;

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
    async fn test_find_info_by_message_id_database_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_group_membership_service = MockGroupMembershipServiceTrait::new();
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        let auth_user_id = 1;
        let message_id = 2;
        let group_chat_id = 3;
        
        let test_message = TextMessageFactory::fake_text_message_with_ids(message_id, auth_user_id, group_chat_id);
        let membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let membership_dto = GroupMembershipReadDto::from(membership);
        
        mock_repo
            .expect_find()
            .with(eq(message_id))
            .times(1)
            .returning({
                let test_message = test_message.clone();
                move |_| Box::pin({
                    let test_message = test_message.clone();
                    async move { Ok(test_message) }
                })
            });

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
            .expect_find_info_by_message_id()
            .with(eq(message_id))
            .times(1)
            .returning(|_| Box::pin(async move { 
                Err(sqlx::Error::Configuration("Database connection failed".into())) 
            }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_group_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_info_by_message_id_internal(auth_user_id, message_id).await;

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