use crate::dto::text_message_dto::TextMessageReadDto;
use crate::dto::text_message_pagination_dto::TextMessagePaginationQuery;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::error::group_chat_error::GroupChatError;
use crate::error::text_message_error::TextMessageError;
use crate::response::paginated_response::{PaginatedResponse, PaginationMetadata};
use crate::response::PaginatedTextMessageResponse;
use crate::service::text_message_service::TextMessageService;

impl TextMessageService {
    pub async fn find_by_group_chat_id_paginated_internal(
        &self,
        group_chat_id: i32,
        pagination_query: TextMessagePaginationQuery,
        auth_user_id: i32,
    ) -> Result<PaginatedTextMessageResponse, ApiError> {
        // Check if the group exists first
        self.group_chat_service
            .find_by_id(group_chat_id)
            .await
            .map_err(|e| match e {
                ApiError::GroupChatError(GroupChatError::GroupChatNotFound) => {
                    ApiError::TextMessageError(TextMessageError::GroupChatNotFound)
                }
                _ => e,
            })?;

        // Check if the auth_user has membership in the group
        self.group_membership_service
            .find_by_user_id_and_group_id(auth_user_id, group_chat_id)
            .await
            .map_err(|e| match e {
                ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                    ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)
                }
                _ => e,
            })?;

        // Il cursor è già validato automaticamente durante la deserializzazione del DTO
        let limit = pagination_query.limit;
        let cursor_datetime = pagination_query.cursor;

        let text_messages = self
            .text_message_repo
            .find_by_group_chat_id_paginated(group_chat_id, cursor_datetime, limit + 1)
            .await
            .map_err(|e| {
                let db_error = match e {
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

        let has_more = text_messages.len() > limit;
        let messages: Vec<_> = text_messages.into_iter().take(limit).collect();

        // Get next cursor from the last message's datetime
        let next_cursor = if has_more && !messages.is_empty() {
            Some(messages.last().unwrap().sent_at)
        } else {
            None
        };

        // Convert to DTOs
        let data: Vec<TextMessageReadDto> = messages
            .into_iter()
            .map(TextMessageReadDto::from)
            .collect();

        let pagination = PaginationMetadata {
            has_more,
            next_cursor,
            page_size: data.len(),
            total_count: None, // We don't calculate total count for cursor-based pagination
        };

        Ok(PaginatedTextMessageResponse {
            data,
            pagination,
        })
    }
}

#[cfg(test)]
mod find_by_group_chat_id_paginated_service_tests {
    use super::*;
    use crate::factory::text_message_factory::TextMessageFactory;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::factory::group_chat_factory::GroupChatFactory;
    use crate::repository::text_message_repository::text_message_repository_trait::MockTextMessageRepositoryTrait;
    use crate::service::group_membership_service::group_membership_service_trait::MockGroupMembershipServiceTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::text_message_service::text_message_service_trait::TextMessageServiceTrait;
    use crate::dto::text_message_pagination_dto::TextMessagePaginationQuery;
    use mockall::predicate::*;
    use std::sync::Arc;
    use crate::dto::group_chat_dto::GroupChatReadDto;
    use crate::dto::group_membership_dto::GroupMembershipReadDto;
    use crate::entity::invitation::Invitation;
    use crate::factory::invitation_factory::InvitationFactory;
    use crate::model::group_membership_model::GroupMembershipWithInvitationRow;
    use crate::utils::mock_database_error::MockDatabaseError;

    // Helper function to create a mock service with successful membership check
    fn create_mock_service_with_membership(
        mock_repo: MockTextMessageRepositoryTrait,
        auth_user_id: i32,
        group_chat_id: i32,
    ) -> TextMessageService {
        let mut mock_membership_service = MockGroupMembershipServiceTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mut membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        membership.user_id = auth_user_id;
        membership.group_chat_id = group_chat_id;
        
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
        
        mock_membership_service
            .expect_find_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_chat_id))
            .returning({
                let membership = membership.clone();
                move |_, _| Box::pin({
                    let membership = membership.clone();
                    async move { Ok(membership.into()) }
                })
            });
        
        TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_membership_service), Arc::new(mock_group_chat_service))
    }

    #[tokio::test]
    async fn test_find_by_group_chat_id_paginated_success() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_membership_service = MockGroupMembershipServiceTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let group_chat_id = 1;
        let auth_user_id = 1;
        let limit = 10;
        let pagination_query = TextMessagePaginationQuery::new(None, limit);
        
        // Mock successful membership check
        let mut group_chat = GroupChatFactory::fake_group_chat();
        group_chat.id = group_chat_id;
        let mut membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        membership.user_id = auth_user_id;
        membership.group_chat_id = group_chat_id;

        mock_group_chat_service
            .expect_find_by_id()
            .with(eq(group_chat_id))
            .returning({
                move |_| Box::pin({
                    let group_chat = group_chat.clone();
                    async move { Ok(GroupChatReadDto::from(group_chat)) }
                })
            });

        mock_membership_service
            .expect_find_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_chat_id))
            .times(1)
            .returning({
                let membership = membership.clone();
                move |_, _| Box::pin({
                    let membership = membership.clone();
                    async move { Ok(GroupMembershipReadDto::from(membership)) }
                })
            });
        
        // Create test messages
        let messages = vec![
            TextMessageFactory::fake_text_message_with_ids(1, 1, group_chat_id),
            TextMessageFactory::fake_text_message_with_ids(2, 2, group_chat_id),
            TextMessageFactory::fake_text_message_with_ids(3, 1, group_chat_id),
        ];
        
        mock_repo
            .expect_find_by_group_chat_id_paginated()
            .with(eq(group_chat_id), eq(None), eq(limit + 1))
            .times(1)
            .returning({
                let messages = messages.clone();
                move |_, _, _| Box::pin({
                    let messages = messages.clone();
                    async move { Ok(messages) }
                })
            });

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_by_group_chat_id_paginated_internal(group_chat_id, pagination_query, auth_user_id).await;

        // Assert
        assert!(result.is_ok());
        let paginated_response = result.unwrap();
        assert_eq!(paginated_response.data.len(), 3);
        assert_eq!(paginated_response.pagination.has_more, false);
        assert_eq!(paginated_response.pagination.next_cursor, None);
        assert_eq!(paginated_response.pagination.page_size, 3);
        
        // Verify all messages belong to the correct group
        for message_dto in &paginated_response.data {
            assert_eq!(message_dto.group_chat_id, group_chat_id);
        }
    }

    #[tokio::test]
    async fn test_find_by_group_chat_id_paginated_unauthorized_user() {
        // Arrange
        let mock_repo = MockTextMessageRepositoryTrait::new();
        let mut mock_membership_service = MockGroupMembershipServiceTrait::new();
        let mut mock_group_chat_service = MockGroupChatServiceTrait::new();
        let group_chat_id = 1;
        let auth_user_id = 999; // User not in the group
        let limit = 10;
        let pagination_query = TextMessagePaginationQuery::new(None, limit);
        
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
            .expect_find_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_chat_id))
            .times(1)
            .returning(|_, _| Box::pin(async move {
                Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound))
            }));

        let service = TextMessageService::new(Arc::new(mock_repo), Arc::new(mock_membership_service), Arc::new(mock_group_chat_service));

        // Act
        let result = service.find_by_group_chat_id_paginated_internal(group_chat_id, pagination_query, auth_user_id).await;

        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::TextMessageError(TextMessageError::UserCannotAccessMessages)));
    }

    #[tokio::test]
    async fn test_find_by_group_chat_id_paginated_with_cursor() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let group_chat_id = 1;
        let limit = 2;
        let cursor = Some(chrono::DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z").unwrap().with_timezone(&chrono::Utc));
        let pagination_query = TextMessagePaginationQuery::new(cursor.clone(), limit);
        
        // Create test messages (limit + 1 to test has_more)
        let messages = vec![
            TextMessageFactory::fake_text_message_with_ids(1, 1, group_chat_id),
            TextMessageFactory::fake_text_message_with_ids(2, 2, group_chat_id),
            TextMessageFactory::fake_text_message_with_ids(3, 1, group_chat_id),
        ];
        
        mock_repo
            .expect_find_by_group_chat_id_paginated()
            .with(eq(group_chat_id), eq(cursor.clone()), eq(limit + 1))
            .times(1)
            .returning({
                let messages = messages.clone();
                move |_, _, _| Box::pin({
                    let messages = messages.clone();
                    async move { Ok(messages) }
                })
            });

        let service = create_mock_service_with_membership(mock_repo, 1, group_chat_id);

        // Act
        let result = service.find_by_group_chat_id_paginated_internal(group_chat_id, pagination_query, 1).await;

        // Assert
        assert!(result.is_ok());
        let paginated_response = result.unwrap();
        assert_eq!(paginated_response.data.len(), 2); // Limited to 2
        assert_eq!(paginated_response.pagination.has_more, true); // 3 messages > 2 limit
        assert!(paginated_response.pagination.next_cursor.is_some());
        assert_eq!(paginated_response.pagination.page_size, 2);
    }

    #[tokio::test]
    async fn test_find_by_group_chat_id_paginated_empty_result() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let group_chat_id = 999;
        let limit = 10;
        let pagination_query = TextMessagePaginationQuery::new(None, limit);
        
        mock_repo
            .expect_find_by_group_chat_id_paginated()
            .with(eq(group_chat_id), eq(None), eq(limit + 1))
            .times(1)
            .returning(move |_, _, _| Box::pin(async move { Ok(vec![]) }));

        let service = create_mock_service_with_membership(mock_repo, 1, group_chat_id);

        // Act
        let result = service.find_by_group_chat_id_paginated_internal(group_chat_id, pagination_query, 1).await;

        // Assert
        assert!(result.is_ok());
        let paginated_response = result.unwrap();
        assert_eq!(paginated_response.data.len(), 0);
        assert_eq!(paginated_response.pagination.has_more, false);
        assert_eq!(paginated_response.pagination.next_cursor, None);
    }

    #[tokio::test]
    async fn test_find_by_group_chat_id_paginated_database_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let group_chat_id = 1;
        let limit = 10;
        let pagination_query = TextMessagePaginationQuery::new(None, limit);
        
        mock_repo
            .expect_find_by_group_chat_id_paginated()
            .with(eq(group_chat_id), eq(None), eq(limit + 1))
            .times(1)
            .returning(|_, _, _| Box::pin(async move { 
                Err(sqlx::Error::Configuration("Database connection failed".into()))
            }));

        let service = create_mock_service_with_membership(mock_repo, 1, group_chat_id);

        // Act
        let result = service.find_by_group_chat_id_paginated_internal(group_chat_id, pagination_query, 1).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(_)) => {
                // Expected error type
            }
            _ => panic!("Expected SomethingWentWrong error"),
        }
    }

    #[tokio::test]
    async fn test_find_by_group_chat_id_paginated_foreign_key_error() {
        // Arrange
        let mut mock_repo = MockTextMessageRepositoryTrait::new();
        let group_chat_id = 1;
        let limit = 10;
        let pagination_query = TextMessagePaginationQuery::new(None, limit);
        
        mock_repo
            .expect_find_by_group_chat_id_paginated()
            .with(eq(group_chat_id), eq(None), eq(limit + 1))
            .times(1)
            .returning(move |_, _, _| Box::pin(async move { 
                Err(MockDatabaseError::foreign_key_violation())
            }));

        let service = create_mock_service_with_membership(mock_repo, 1, group_chat_id);

        // Act
        let result = service.find_by_group_chat_id_paginated_internal(group_chat_id, pagination_query, 1).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::ForeignKeyViolation(_)) => {
                // Expected error type
            }
            _ => panic!("Expected ForeignKeyViolation error"),
        }
    }
}
