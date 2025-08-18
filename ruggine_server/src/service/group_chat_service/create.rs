use crate::entity::group_chat::{NewGroupChat};
use crate::error::{api_error::ApiError, db_error::DbError};
use crate::dto::group_chat_dto::{GroupChatCreateDto, GroupChatReadDto};
use crate::dto::invitation_dto::{InvitationCreateDto, InvitationUpdateStatusDto};
use crate::entity::invitation::InvitationStatus;
use crate::entity::group_membership::MemberRole;
use crate::service::group_chat_service::GroupChatService;

impl GroupChatService {
    pub async fn create_internal(&self, payload: GroupChatCreateDto, created_by: i32) -> Result<GroupChatReadDto, ApiError> {
        // Create the new group chat entity
        let new_group_chat = NewGroupChat {
            name: payload.name.clone(),
            description: payload.description.clone(),
            created_by,
        };

        // Insert the group chat into the database
        let group_id = match self.group_chat_repo.insert(new_group_chat).await {
            Ok(id) => id,
            Err(sqlx_error) => return match sqlx_error {
                sqlx::Error::Database(db_err) => {
                    if let Some(code) = db_err.code() {
                        match code.as_ref() {
                            "23505" => {
                                // Unique violation
                                Err(ApiError::DbError(DbError::UniqueConstraintViolation(db_err.to_string())))
                            },
                            "23503" => {
                                // Foreign key violation (es. created_by non esiste)
                                Err(ApiError::DbError(DbError::ForeignKeyViolation(db_err.to_string())))
                            },
                            _ => {
                                Err(ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string())))
                            }
                        }
                    } else {
                        Err(ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string())))
                    }
                },
                _ => Err(ApiError::DbError(DbError::SomethingWentWrong(sqlx_error.to_string()))),
            },
        };

        // Create an invitation for the creator to join their own group
        let invitation_create_dto = InvitationCreateDto {
            to_user_id: created_by,
            group_chat_id: group_id,
            role_at_join: MemberRole::Admin,
        };

        let invitation_service = self.invitation_service();

        let invitation = match invitation_service.send_for_group_chat_create(invitation_create_dto, created_by).await {
            Ok(invitation) => invitation,
            Err(e) => return Err(e),
        };

        // Accept the invitation automatically
        let invitation_update_dto = InvitationUpdateStatusDto {
            invitation_id: invitation.id,
            status: InvitationStatus::Accepted,
        };

        if let Err(e) = invitation_service.update_status(invitation_update_dto, created_by).await {
            eprintln!("Errore durante update_status: {:?}", e); // stampa l'errore completo
            return Err(e);
        }

        // Return the created group chat as DTO
        let now = chrono::Utc::now();
        Ok(GroupChatReadDto {
            id: group_id,
            name: payload.name,
            description: payload.description,
            created_by,
            created_at: now,
            updated_at: now,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::group_chat_repository::group_chat_repository_trait::MockGroupChatRepositoryTrait;
    use crate::service::group_chat_service::GroupChatServiceTrait;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use crate::service::invitation_service::invitation_service_trait::MockInvitationServiceTrait;
    use crate::factory::group_chat_factory::GroupChatFactory;
    use crate::factory::user_factory::UserFactory;
    use crate::factory::invitation_factory::InvitationFactory;
    use crate::utils::mock_database_error::MockDatabaseError;
    use std::sync::Arc;

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_success() {
        // Arrange: Set up mock repositories with successful responses
        let mut mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mut mock_invitation_service = MockInvitationServiceTrait::new();

        let user = UserFactory::fake_user();
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("test");
        let created_by = user.id;
        let expected_group_id = 1;

        // Mock: group_chat_repo.insert should return the group ID
        mock_group_repo
            .expect_insert()
            .returning(move |_| {
                Box::pin(async move { Ok(expected_group_id) })
            });

        // Mock: invitation_service.send_for_group_chat_create should return a mock invitation
        let mock_invitation = InvitationFactory::fake_invitation_read_dto();
        let invitation_clone = mock_invitation.clone();
        mock_invitation_service
            .expect_send_for_group_chat_create()
            .returning(move |_, _| {
                Box::pin({
                    let value = invitation_clone.clone();
                    async move { Ok(value.clone()) }
                })
            });

        // Mock: invitation_service.update_status should return success
        let mock_update_response = crate::dto::invitation_dto::InvitationUpdateResponseDto {
            id: mock_invitation.id,
            status: InvitationStatus::Accepted,
            responded_at: chrono::Utc::now(),
            group_membership_id: Some(1),
        };
        let update_response_clone = mock_update_response.clone();
        mock_invitation_service
            .expect_update_status()
            .returning(move |_, _| {
                Box::pin({
                    let value = update_response_clone.clone();
                    async move { Ok(value.clone()) }
                })
            });

        let service = GroupChatService::with(
            Arc::new(mock_group_repo),
            Arc::new(mock_user_service),
        );
        service.set_invitation_service(Arc::new(mock_invitation_service));

        // Act: Call the create_group_chat method
        let result = service.create_internal(create_dto.clone(), created_by).await;

        // Assert: Verify the group chat was created successfully
        assert!(result.is_ok(), "Failed to create group chat: {:?}", result);
        let group_dto = result.unwrap();
        assert_eq!(group_dto.id, expected_group_id);
        assert_eq!(group_dto.name, create_dto.name);
        assert_eq!(group_dto.description, create_dto.description);
        assert_eq!(group_dto.created_by, created_by);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_unique_constraint_violation() {
        // Arrange: Set up mocks where group insertion fails with unique constraint violation
        let mut mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_invitation_service = MockInvitationServiceTrait::new();

        let user = UserFactory::fake_user();
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("test");
        let created_by = user.id;

        // Mock: group_chat_repo.insert should return unique constraint violation
        mock_group_repo
            .expect_insert()
            .returning(move |_| {
                let db_error = sqlx::Error::Database(Box::new(MockDatabaseError::new("23505".to_string())));
                Box::pin(async move { Err(db_error) })
            });

        let service = GroupChatService::with(
            Arc::new(mock_group_repo),
            Arc::new(mock_user_service),
        );
        service.set_invitation_service(Arc::new(mock_invitation_service));

        // Act: Call the create_group_chat method
        let result = service.create_internal(create_dto, created_by).await;

        // Assert: Verify the unique constraint violation error is returned
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::UniqueConstraintViolation(_)) => {
                // Expected error
            }
            other => panic!("Expected UniqueConstraintViolation error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_database_error() {
        // Arrange: Set up mocks where group insertion fails with general database error
        let mut mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mock_invitation_service = MockInvitationServiceTrait::new();

        let user = UserFactory::fake_user();
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("test");
        let created_by = user.id;

        // Mock: group_chat_repo.insert should return a general database error
        mock_group_repo
            .expect_insert()
            .returning(move |_| {
                let db_error = sqlx::Error::PoolClosed;
                Box::pin(async move { Err(db_error) })
            });

        let service = GroupChatService::with(
            Arc::new(mock_group_repo),
            Arc::new(mock_user_service),
        );
        service.set_invitation_service(Arc::new(mock_invitation_service));

        // Act: Call the create_group_chat method
        let result = service.create_internal(create_dto, created_by).await;

        // Assert: Verify the database error is returned
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(_)) => {
                // Expected error
            }
            other => panic!("Expected SomethingWentWrong error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_invitation_send_fails() {
        // Arrange: Set up mocks where invitation sending fails
        let mut mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mut mock_invitation_service = MockInvitationServiceTrait::new();

        let user = UserFactory::fake_user();
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("test");
        let created_by = user.id;
        let expected_group_id = 1;

        // Mock: group_chat_repo.insert should succeed
        mock_group_repo
            .expect_insert()
            .returning(move |_| {
                Box::pin(async move { Ok(expected_group_id) })
            });

        // Mock: invitation_service.send_for_group_chat_create should fail
        mock_invitation_service
            .expect_send_for_group_chat_create()
            .returning(move |_, _| {
                Box::pin(async move { Err(ApiError::DbError(DbError::SomethingWentWrong("Invitation send failed".to_string()))) })
            });

        let service = GroupChatService::with(
            Arc::new(mock_group_repo),
            Arc::new(mock_user_service),
        );
        service.set_invitation_service(Arc::new(mock_invitation_service));

        // Act: Call the create_group_chat method
        let result = service.create_internal(create_dto, created_by).await;

        // Assert: Verify the invitation error is returned
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(msg)) => {
                assert!(msg.contains("Invitation send failed"));
            }
            other => panic!("Expected SomethingWentWrong error, got: {:?}", other),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_invitation_accept_fails() {
        // Arrange: Set up mocks where invitation acceptance fails
        let mut mock_group_repo = MockGroupChatRepositoryTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        let mut mock_invitation_service = MockInvitationServiceTrait::new();

        let user = UserFactory::fake_user();
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("test");
        let created_by = user.id;
        let expected_group_id = 1;

        // Mock: group_chat_repo.insert should succeed
        mock_group_repo
            .expect_insert()
            .returning(move |_| {
                Box::pin(async move { Ok(expected_group_id) })
            });

        // Mock: invitation_service.send_for_group_chat_create should succeed
        let mock_invitation = InvitationFactory::fake_invitation_read_dto();
        let invitation_clone = mock_invitation.clone();
        mock_invitation_service
            .expect_send_for_group_chat_create()
            .returning(move |_, _| {
                Box::pin({
                    let value = invitation_clone.clone();
                    async move { Ok(value.clone()) }
                })
            });

        // Mock: invitation_service.update_status should fail
        mock_invitation_service
            .expect_update_status()
            .returning(move |_, _| {
                Box::pin(async move { Err(ApiError::DbError(DbError::SomethingWentWrong("Invitation accept failed".to_string()))) })
            });

        let service = GroupChatService::with(
            Arc::new(mock_group_repo),
            Arc::new(mock_user_service),
        );
        service.set_invitation_service(Arc::new(mock_invitation_service));

        // Act: Call the create_group_chat method
        let result = service.create_internal(create_dto, created_by).await;

        // Assert: Verify the invitation error is returned
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(msg)) => {
                assert!(msg.contains("Invitation accept failed"));
            }
            other => panic!("Expected SomethingWentWrong error, got: {:?}", other),
        }
    }
}

