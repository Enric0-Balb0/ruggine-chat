use crate::error::{api_error::ApiError, db_error::DbError, group_membership_error::GroupMembershipError};
use crate::dto::group_membership_dto::{GroupMembershipCreateDto, GroupMembershipReadDto};
use crate::error::invitation_error::InvitationError;
use crate::service::group_membership_service::GroupMembershipService;

impl GroupMembershipService {
    pub async fn create_checked_internal(&self, payload: GroupMembershipCreateDto, auth_user_id: i32) -> Result<GroupMembershipReadDto, ApiError> {
        // Verify invitation exists
        match self
            .invitation_service()
            .find_by_id_and_user_id(payload.invitation_id, auth_user_id)
            .await
        {
            Ok(_) => {

            },
            Err(ApiError::InvitationError(InvitationError::InvitationNotFound)) => {
                return Err(ApiError::GroupMembershipError(
                    GroupMembershipError::InvitationNotFound,
                ))
            },
            Err(e) => {
                return Err(ApiError::DbError(
                    DbError::SomethingWentWrong(e.to_string()),
                ))
            }
        };

        // Create the new group membership entity
        let new_group_membership = payload.to_new_group_membership();

        // Insert the group membership into the database
        let membership_id = match self.group_membership_repo.insert(new_group_membership).await {
            Ok(id) => id,
            Err(sqlx_error) => return match sqlx_error {
                sqlx::Error::Database(db_err) => {
                    if let Some(code) = db_err.code() {
                        match code.as_ref() {
                            "23505" => {
                                // Unique violation - user already in group (invitation_id is unique)
                                Err(ApiError::GroupMembershipError(GroupMembershipError::UserAlreadyInGroup))
                            },
                            "23503" => {
                                // Foreign key violation
                                Err(ApiError::DbError(DbError::ForeignKeyViolation(db_err.to_string())))
                            },
                            "P0001" => {
                                // Custom trigger error: Only one active membership per user per group_chat
                                if db_err.message().contains("Only one active membership allowed") {
                                    Err(ApiError::GroupMembershipError(GroupMembershipError::UserAlreadyInGroup))
                                } else {
                                    Err(ApiError::DbError(DbError::SomethingWentWrong(db_err.to_string())))
                                }
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

        // Retrieve the newly created membership to return it as a DTO
        let membership = match self
            .group_membership_repo
            .find_by_id_and_user_id(membership_id, auth_user_id)
            .await
        {
            Ok(membership) => membership,
            Err(_) => {
                return Err(ApiError::GroupMembershipError(
                    GroupMembershipError::GroupMembershipNotFound,
                ))
            }
        };

        Ok(GroupMembershipReadDto::from(membership))
    }
}

#[cfg(test)]
mod group_membership_service_create_checked_tests {
    use super::*;
    use mockall::predicate::*;
    use crate::entity::group_membership::{MemberRole, MembershipStatus};
    use crate::service::group_membership_service::group_membership_service_trait::GroupMembershipServiceTrait;
    use crate::factory::invitation_factory::InvitationFactory;
    use chrono::Utc;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use crate::service::invitation_service::invitation_service_trait::MockInvitationServiceTrait;
    use std::sync::{Arc};
    use crate::model::group_membership_model::GroupMembershipWithInvitationRow;

    fn create_service_with_mock_invitation_service(
        mock_group_membership_repo: MockGroupMembershipRepositoryTrait,
        mock_invitation_service: MockInvitationServiceTrait,
    ) -> GroupMembershipService {
        let mock_group_chat_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();
        
        let service = GroupMembershipService::with(
            Arc::new(mock_group_membership_repo),
            Arc::new(mock_group_chat_service),
            Arc::new(mock_user_service),
        );
        
        service.set_invitation_service(Arc::new(mock_invitation_service));
        service
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_checked_internal_success() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_invitation_service = MockInvitationServiceTrait::new();

        let auth_user_id = 1;
        let invitation_id = 100;
        
        let create_dto = GroupMembershipCreateDto {
            invitation_id,
            role: MemberRole::Member,
        };

        // Mock invitation exists and is valid
        let mut invitation_read_dto = InvitationFactory::fake_invitation_read_dto();
        invitation_read_dto.id = invitation_id;
        invitation_read_dto.to_user_id = auth_user_id;

        mock_invitation_service
            .expect_find_by_id_and_user_id()
            .with(eq(invitation_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                let invitation = invitation_read_dto.clone();
                Box::pin(async move { Ok(invitation) })
            });

        // Mock successful membership insertion
        let membership_id = 50;
        mock_group_membership_repo
            .expect_insert()
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Ok(membership_id) })
            });

        // Mock successful membership retrieval
        let expected_membership_row = GroupMembershipWithInvitationRow {
            id: membership_id,
            role: MemberRole::Member,
            joined_at: Utc::now(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id,
            user_id: auth_user_id,
            group_chat_id: 1,
        };

        mock_group_membership_repo
            .expect_find_by_id_and_user_id()
            .with(eq(membership_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                let membership = expected_membership_row.clone();
                Box::pin(async move { Ok(membership) })
            });

        let service = create_service_with_mock_invitation_service(
            mock_group_membership_repo,
            mock_invitation_service,
        );

        // Act
        let result = service.create_checked_internal(create_dto, auth_user_id).await;

        // Assert
        assert!(result.is_ok());
        let membership_read_dto = result.unwrap();
        assert_eq!(membership_read_dto.id, membership_id);
        assert_eq!(membership_read_dto.invitation_id, invitation_id);
        assert_eq!(membership_read_dto.user_id, auth_user_id);
        assert_eq!(membership_read_dto.role, MemberRole::Member);
        assert_eq!(membership_read_dto.membership_status, MembershipStatus::Active);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_checked_internal_invitation_not_found() {
        // Arrange
        let mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_invitation_service = MockInvitationServiceTrait::new();

        let auth_user_id = 1;
        let invitation_id = 999;
        
        let create_dto = GroupMembershipCreateDto {
            invitation_id,
            role: MemberRole::Member,
        };

        // Mock invitation not found
        mock_invitation_service
            .expect_find_by_id_and_user_id()
            .with(eq(invitation_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(ApiError::InvitationError(InvitationError::InvitationNotFound)) })
            });

        let service = create_service_with_mock_invitation_service(
            mock_group_membership_repo,
            mock_invitation_service,
        );

        // Act
        let result = service.create_checked_internal(create_dto, auth_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::InvitationNotFound) => {
                // Expected error
            }
            e => panic!("Expected InvitationNotFound error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_checked_internal_invitation_service_error() {
        // Arrange
        let mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_invitation_service = MockInvitationServiceTrait::new();

        let auth_user_id = 1;
        let invitation_id = 100;
        
        let create_dto = GroupMembershipCreateDto {
            invitation_id,
            role: MemberRole::Member,
        };

        // Mock invitation service database error
        mock_invitation_service
            .expect_find_by_id_and_user_id()
            .with(eq(invitation_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(ApiError::DbError(DbError::SomethingWentWrong("Database connection failed".to_string()))) })
            });

        let service = create_service_with_mock_invitation_service(
            mock_group_membership_repo,
            mock_invitation_service,
        );

        // Act
        let result = service.create_checked_internal(create_dto, auth_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(_)) => {
                // Expected error
            }
            e => panic!("Expected DbError::SomethingWentWrong error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_checked_internal_user_already_in_group_unique_violation() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_invitation_service = MockInvitationServiceTrait::new();

        let auth_user_id = 1;
        let invitation_id = 100;
        
        let create_dto = GroupMembershipCreateDto {
            invitation_id,
            role: MemberRole::Member,
        };

        // Mock invitation exists and is valid
        let mut invitation_read_dto = InvitationFactory::fake_invitation_read_dto();
        invitation_read_dto.id = invitation_id;
        invitation_read_dto.to_user_id = auth_user_id;

        mock_invitation_service
            .expect_find_by_id_and_user_id()
            .with(eq(invitation_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                let invitation = invitation_read_dto.clone();
                Box::pin(async move { Ok(invitation) })
            });

        // Mock unique constraint violation (user already in group)
        mock_group_membership_repo
            .expect_insert()
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Err(sqlx::Error::Database(Box::new(crate::utils::mock_database_error::MockDatabaseError::new("23505".to_string())))) })
            });

        let service = create_service_with_mock_invitation_service(
            mock_group_membership_repo,
            mock_invitation_service,
        );

        // Act
        let result = service.create_checked_internal(create_dto, auth_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::UserAlreadyInGroup) => {
                // Expected error
            }
            e => panic!("Expected UserAlreadyInGroup error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_checked_internal_foreign_key_violation() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_invitation_service = MockInvitationServiceTrait::new();

        let auth_user_id = 1;
        let invitation_id = 100;
        
        let create_dto = GroupMembershipCreateDto {
            invitation_id,
            role: MemberRole::Member,
        };

        // Mock invitation exists and is valid
        let mut invitation_read_dto = InvitationFactory::fake_invitation_read_dto();
        invitation_read_dto.id = invitation_id;
        invitation_read_dto.to_user_id = auth_user_id;

        mock_invitation_service
            .expect_find_by_id_and_user_id()
            .with(eq(invitation_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                let invitation = invitation_read_dto.clone();
                Box::pin(async move { Ok(invitation) })
            });

        // Mock foreign key constraint violation
        mock_group_membership_repo
            .expect_insert()
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Err(sqlx::Error::Database(Box::new(crate::utils::mock_database_error::MockDatabaseError::new("23503".to_string())))) })
            });

        let service = create_service_with_mock_invitation_service(
            mock_group_membership_repo,
            mock_invitation_service,
        );

        // Act
        let result = service.create_checked_internal(create_dto, auth_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::ForeignKeyViolation(_)) => {
                // Expected error
            }
            e => panic!("Expected ForeignKeyViolation error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_checked_internal_custom_trigger_error() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_invitation_service = MockInvitationServiceTrait::new();

        let auth_user_id = 1;
        let invitation_id = 100;
        
        let create_dto = GroupMembershipCreateDto {
            invitation_id,
            role: MemberRole::Member,
        };

        // Mock invitation exists and is valid
        let mut invitation_read_dto = InvitationFactory::fake_invitation_read_dto();
        invitation_read_dto.id = invitation_id;
        invitation_read_dto.to_user_id = auth_user_id;

        mock_invitation_service
            .expect_find_by_id_and_user_id()
            .with(eq(invitation_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                let invitation = invitation_read_dto.clone();
                Box::pin(async move { Ok(invitation) })
            });

        // Mock custom trigger error that contains the specific message
        mock_group_membership_repo
            .expect_insert()
            .times(1)
            .returning(move |_| {
                // Create a mock database error with P0001 code and the specific message
                let mock_error = crate::utils::mock_database_error::MockDatabaseError::with_message(
                    "P0001".to_string(),
                    "Only one active membership allowed per user per group".to_string()
                );
                Box::pin(async move { Err(sqlx::Error::Database(Box::new(mock_error))) })
            });

        let service = create_service_with_mock_invitation_service(
            mock_group_membership_repo,
            mock_invitation_service,
        );

        // Act
        let result = service.create_checked_internal(create_dto, auth_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::UserAlreadyInGroup) => {
                // Expected error
            }
            e => panic!("Expected UserAlreadyInGroup error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_checked_internal_generic_database_error() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_invitation_service = MockInvitationServiceTrait::new();

        let auth_user_id = 1;
        let invitation_id = 100;
        
        let create_dto = GroupMembershipCreateDto {
            invitation_id,
            role: MemberRole::Member,
        };

        // Mock invitation exists and is valid
        let mut invitation_read_dto = InvitationFactory::fake_invitation_read_dto();
        invitation_read_dto.id = invitation_id;
        invitation_read_dto.to_user_id = auth_user_id;

        mock_invitation_service
            .expect_find_by_id_and_user_id()
            .with(eq(invitation_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                let invitation = invitation_read_dto.clone();
                Box::pin(async move { Ok(invitation) })
            });

        // Mock generic database error
        mock_group_membership_repo
            .expect_insert()
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Err(sqlx::Error::Database(Box::new(crate::utils::mock_database_error::MockDatabaseError::new("42000".to_string())))) })
            });

        let service = create_service_with_mock_invitation_service(
            mock_group_membership_repo,
            mock_invitation_service,
        );

        // Act
        let result = service.create_checked_internal(create_dto, auth_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(_)) => {
                // Expected error
            }
            e => panic!("Expected DbError::SomethingWentWrong error, got: {:?}", e),
        }
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_checked_internal_membership_retrieval_failure() {
        // Arrange
        let mut mock_group_membership_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_invitation_service = MockInvitationServiceTrait::new();

        let auth_user_id = 1;
        let invitation_id = 100;
        
        let create_dto = GroupMembershipCreateDto {
            invitation_id,
            role: MemberRole::Member,
        };

        // Mock invitation exists and is valid
        let mut invitation_read_dto = InvitationFactory::fake_invitation_read_dto();
        invitation_read_dto.id = invitation_id;
        invitation_read_dto.to_user_id = auth_user_id;

        mock_invitation_service
            .expect_find_by_id_and_user_id()
            .with(eq(invitation_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                let invitation = invitation_read_dto.clone();
                Box::pin(async move { Ok(invitation) })
            });

        // Mock successful membership insertion
        let membership_id = 50;
        mock_group_membership_repo
            .expect_insert()
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Ok(membership_id) })
            });

        // Mock failed membership retrieval
        mock_group_membership_repo
            .expect_find_by_id_and_user_id()
            .with(eq(membership_id), eq(auth_user_id))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(sqlx::Error::RowNotFound) })
            });

        let service = create_service_with_mock_invitation_service(
            mock_group_membership_repo,
            mock_invitation_service,
        );

        // Act
        let result = service.create_checked_internal(create_dto, auth_user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            e => panic!("Expected GroupMembershipNotFound error, got: {:?}", e),
        }
    }
}
