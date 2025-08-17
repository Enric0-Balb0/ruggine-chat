use crate::dto::text_message_dto::{TextMessageCreateDto, TextMessageReadDto};
use crate::entity::group_membership::MembershipStatus;
use crate::error::api_error::ApiError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::group_chat_error::GroupChatError;
use crate::error::text_message_error::TextMessageError;
use crate::error::db_error::DbError;
use crate::entity::text_message::NewTextMessage;
use crate::service::text_message_service::TextMessageService;

impl TextMessageService {
    pub async fn create_internal(
        &self,
        payload: TextMessageCreateDto,
        sender_id: i32,
    ) -> Result<TextMessageReadDto, ApiError> {
        // Check if the group exists first
        self.group_chat_service
            .find_by_id(payload.group_chat_id)
            .await
            .map_err(|e| match e {
                ApiError::GroupChatError(GroupChatError::GroupChatNotFound) => {
                    ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)
                }
                _ => e,
            })?;

        // Check if the sender has active membership in the group
        let membership = self.group_membership_service
            .find_active_by_user_id_and_group_id(sender_id, payload.group_chat_id)
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

        // Create the new text message entity
        let new_text_message = NewTextMessage {
            content: payload.content.clone(),
            sender_id,
            group_chat_id: payload.group_chat_id,
        };

        // Insert the message into the database
        let message_id = self
            .text_message_repo
            .insert(new_text_message)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(db_err) => {
                    if let Some(code) = db_err.code() {
                        if code == "23503" {
                            return ApiError::DbError(DbError::ForeignKeyViolation(db_err.to_string()));
                        }
                    }
                    ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string()))
                }
                _ => ApiError::DbError(DbError::SomethingWentWrong(e.to_string())),
            })?;

        // Retrieve the created message to return it
        self.text_message_repo
            .find(message_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
                    ApiError::TextMessageError(TextMessageError::MessageNotFound)
                }
                sqlx::Error::Database(db_err) => {
                    ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string()))
                }
                _ => ApiError::DbError(DbError::SomethingWentWrong(e.to_string())),
            })
            .map(TextMessageReadDto::from)
    }
}

#[cfg(test)]
mod create_service_tests {
    use super::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::factory::group_chat_factory::GroupChatFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::dto::group_membership_dto::GroupMembershipReadDto;
    use mockall::predicate::*;
    use std::sync::Arc;
    use crate::utils::mock_database_error::MockDatabaseError;

    // Helper function to create a mock service with successful validations
    fn create_mock_service_with_validations(
        mock_repo: MockTextMessageRepositoryTrait,
        sender_id: i32,
        group_chat_id: i32,
    ) -> TextMessageService {
        let mut mock_membership_service = MockGroupMembershipServiceTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        
        // Mock successful group chat lookup
        let group_chat = GroupChatFactory::fake_group_chat_read_dto();
        mock_group_chat_service
            .expect_find_by_id()
            .with(eq(group_chat_id))
            .returning({
                let group_chat = group_chat.clone();
                move |_| Box::pin({
                    let group_chat = group_chat.clone();
                    async move { Ok(group_chat) }
                })
            });
        
        // Mock successful membership check
        let mut membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        membership.user_id = sender_id;
        membership.group_chat_id = group_chat_id;
        
        mock_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(sender_id), eq(group_chat_id))
            .returning({
                let membership = membership.clone();
                move |_, _| Box::pin({
                    let membership = membership.clone();
                    async move { Ok(GroupMembershipReadDto::from(membership)) }
                })
            });

        TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_membership_service), Arc::new(mock_group_chat_service))
    }

    #[tokio::test]
    async fn test_create_message_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let sender_id = 1;
        let group_chat_id = 1;
        let message_id = 1;
        let payload = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat_id);
        let expected_message = TextMessageFactory::fake_text_message_with_ids(message_id, sender_id, group_chat_id);
        
        // Mock successful insert
        mock_repo
            .expect_insert()
            .with(function({
                let payload = payload.clone();
                move |new_msg: &NewTextMessage| {
                    new_msg.content == payload.content && 
                    new_msg.sender_id == sender_id && 
                    new_msg.group_chat_id == group_chat_id
                }
            }))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(message_id) }));
        
        // Mock successful find after insert
        mock_repo
            .expect_find()
            .with(eq(message_id))
            .times(1)
            .returning({
                let expected_message = expected_message.clone();
                move |_| Box::pin({
                    let expected_message = expected_message.clone();
                    async move { Ok(expected_message) }
                })
            });

        let service = create_mock_service_with_validations(mock_repo, sender_id, group_chat_id);

        // Act
        let result = service.create_internal(payload.clone(), sender_id).await;

        // Assert
        assert!(result.is_ok());
        let created_message = result.unwrap();
        assert_eq!(created_message.id, message_id);
        assert_eq!(created_message.content, payload.content);
        assert_eq!(created_message.sender_id, sender_id);
        assert_eq!(created_message.group_chat_id, group_chat_id);
    }

    #[tokio::test]
    async fn test_create_message_group_not_found() {
        // Arrange
        let mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_membership_service = MockGroupMembershipServiceTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let sender_id = 1;
        let group_chat_id = 999; // Non-existent group
        let payload = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat_id);
        
        // Mock failed group chat lookup
        mock_group_chat_service
            .expect_find_by_id()
            .with(eq(group_chat_id))
            .times(1)
            .returning(|_| Box::pin(async move {
                Err(ApiError::GroupChatError(GroupChatError::GroupChatNotFound))
            }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.create_internal(payload, sender_id).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)));
    }

    #[tokio::test]
    async fn test_create_message_user_not_member() {
        // Arrange
        let mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_membership_service = MockGroupMembershipServiceTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let sender_id = 999; // User not in group
        let group_chat_id = 1;
        let payload = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat_id);
        
        // Mock successful group chat lookup
        let group_chat = GroupChatFactory::fake_group_chat_read_dto();
        mock_group_chat_service
            .expect_find_by_id()
            .with(eq(group_chat_id))
            .returning({
                let group_chat = group_chat.clone();
                move |_| Box::pin({
                    let group_chat = group_chat.clone();
                    async move { Ok(group_chat) }
                })
            });
        
        // Mock failed membership check
        mock_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(sender_id), eq(group_chat_id))
            .times(1)
            .returning(|_, _| Box::pin(async move {
                Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound))
            }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.create_internal(payload, sender_id).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)));
    }

    #[tokio::test]
    async fn test_create_message_user_not_active_membership() {
        // Arrange
        let mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_membership_service = MockGroupMembershipServiceTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let sender_id = 1;
        let group_chat_id = 1;
        let payload = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat_id);

        // Mock successful group chat lookup
        let group_chat = GroupChatFactory::fake_group_chat_read_dto();
        mock_group_chat_service
            .expect_find_by_id()
            .with(eq(group_chat_id))
            .returning({
                let group_chat = group_chat.clone();
                move |_| Box::pin({
                    let group_chat = group_chat.clone();
                    async move { Ok(group_chat) }
                })
            });

        // Mock failed membership check
        let mut membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        membership.user_id = sender_id;
        membership.group_chat_id = group_chat_id;
        membership.membership_status = MembershipStatus::Left;

        mock_membership_service
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(sender_id), eq(group_chat_id))
            .returning({
                let membership = membership.clone();
                move |_, _| Box::pin({
                    let membership = membership.clone();
                    async move { Ok(GroupMembershipReadDto::from(membership)) }
                })
            });

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.create_internal(payload, sender_id).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)));
    }

    #[tokio::test]
    async fn test_create_message_insert_fails() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let sender_id = 1;
        let group_chat_id = 1;
        let payload = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat_id);

        // Mock failed insert
        mock_repo
            .expect_insert()
            .times(1)
            .returning(|_| Box::pin(async move { 
                Err(MockDatabaseError::foreign_key_violation()) 
            }));

        let service = create_mock_service_with_validations(mock_repo, sender_id, group_chat_id);

        // Act
        let result = service.create_internal(payload, sender_id).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::DbError(DbError::ForeignKeyViolation(_))));
    }

    #[tokio::test]
    async fn test_create_message_find_after_insert_fails() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let sender_id = 1;
        let group_chat_id = 1;
        let message_id = 1;
        let payload = TextMessageFactory::fake_text_message_create_dto_with_group_id(group_chat_id);

        // Mock successful insert
        mock_repo
            .expect_insert()
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(message_id) }));
        
        // Mock failed find after insert
        mock_repo
            .expect_find()
            .with(eq(message_id))
            .times(1)
            .returning(|_| Box::pin(async move { Err(sqlx::Error::RowNotFound) }));

        let service = create_mock_service_with_validations(mock_repo, sender_id, group_chat_id);

        // Act
        let result = service.create_internal(payload, sender_id).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::TextMessageError(TextMessageError::MessageNotFound)));
    }
}
