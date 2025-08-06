use crate::error::{api_error::ApiError, db_error::DbError, group_membership_error::GroupMembershipError};
use crate::dto::group_membership_dto::{GroupMembershipCreateDto, GroupMembershipReadDto};
use crate::entity::invitation::InvitationStatus;
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
            Ok(invitation) => {

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

/*
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use mockall::predicate::*;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use crate::service::invitation_service::invitation_service_trait::MockInvitationServiceTrait;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::factory::user_factory::UserFactory;
    use crate::dto::group_chat_dto::GroupChatReadDto;
    use crate::error::group_chat_error::GroupChatError;
    use crate::utils::mock_database_error::MockDatabaseError;
    use chrono::Utc;

    fn setup_service(
        mock_repo: MockGroupMembershipRepositoryTrait,
        mock_group_service: MockGroupChatServiceTrait,
        mock_user_service: MockUserServiceTrait,
    ) -> GroupMembershipService {
        GroupMembershipService::with(
            Arc::new(mock_repo),
            Arc::new(mock_group_service),
            Arc::new(mock_user_service),
        )
    }

    #[tokio::test]
    async fn test_create_internal_success() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_group_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();

        let create_dto = GroupMembershipFactory::fake_create_group_membership_dto();
        let expected_membership = GroupMembershipFactory::fake_group_membership();
        let new_membership = create_dto.to_new_group_membership();

        mock_group_service.expect_find_by_id()
            .with(eq(create_dto.group_chat_id))
            .returning(|_| Box::pin(async {
                Ok(GroupChatReadDto {
                    id: 1,
                    name: "Test Group".to_string(),
                    description: "Test Description".to_string(),
                    created_by: 1,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
            }));

        mock_user_service.expect_find_by_id()
            .with(eq(create_dto.invitation_id))
            .returning(|_| Box::pin(async { Ok(UserFactory::fake_read_user_dto()) }));

        mock_repo.expect_insert()
            .with(eq(new_membership.clone()))
            .returning(|_| Box::pin(async { Ok(1) }));

        let expected_membership_cloned = expected_membership.clone();

        mock_repo
            .expect_find_by_id_and_user_id()
            .with(eq(1), eq(create_dto.invitation_id))
            .returning(move |_, _| {
                let membership = expected_membership_cloned.clone();
                Box::pin(async move { Ok(membership) })
            });

        let service = setup_service(mock_repo, mock_group_service, mock_user_service);

        let result = service.create_checked_internal(create_dto.clone()).await;

        assert!(result.is_ok());
        let membership_dto = result.unwrap();
        assert_eq!(membership_dto.user_id, create_dto.invitation_id);
        assert_eq!(membership_dto.group_chat_id, create_dto.group_chat_id);
        assert_eq!(membership_dto.id, 1);
    }

    #[tokio::test]
    async fn test_create_internal_group_not_found() {
        let mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_group_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        let create_dto = GroupMembershipFactory::fake_create_group_membership_dto();

        mock_group_service.expect_find_by_id()
            .with(eq(create_dto.group_chat_id))
            .returning(|_| Box::pin(async { Err(ApiError::GroupChatError(GroupChatError::GroupChatNotFound)) }));

        let service = setup_service(mock_repo, mock_group_service, mock_user_service);

        let result = service.create_checked_internal(create_dto).await;

        assert!(matches!(
            result,
            Err(ApiError::GroupMembershipError(GroupMembershipError::GroupNotFound))
        ));
    }

    #[tokio::test]
    async fn test_create_internal_user_not_found() {
        let mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_group_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();

        let create_dto = GroupMembershipFactory::fake_create_group_membership_dto();

        mock_group_service.expect_find_by_id()
            .with(eq(create_dto.group_chat_id))
            .returning(|_| Box::pin(async {
                Ok(GroupChatReadDto {
                    id: 1,
                    name: "Test Group".to_string(),
                    description: "Test Description".to_string(),
                    created_by: 1,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
            }));

        mock_user_service.expect_find_by_id()
            .with(eq(create_dto.invitation_id))
            .returning(|_| Box::pin(async {
                Err(ApiError::DbError(DbError::SomethingWentWrong("User not found".to_string())))
            }));

        let service = setup_service(mock_repo, mock_group_service, mock_user_service);

        let result = service.create_checked_internal(create_dto).await;

        assert!(matches!(result, Err(ApiError::DbError(_))));
    }

    #[tokio::test]
    async fn test_create_internal_user_already_in_group() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_group_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();

        let create_dto = GroupMembershipFactory::fake_create_group_membership_dto();
        let new_membership = create_dto.to_new_group_membership();

        mock_group_service.expect_find_by_id()
            .with(eq(create_dto.group_chat_id))
            .returning(|_| Box::pin(async {
                Ok(GroupChatReadDto {
                    id: 1,
                    name: "Test Group".to_string(),
                    description: "Test Description".to_string(),
                    created_by: 1,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
            }));

        mock_user_service.expect_find_by_id()
            .with(eq(create_dto.invitation_id))
            .returning(|_| Box::pin(async { Ok(UserFactory::fake_read_user_dto()) }));

        mock_repo.expect_insert()
            .with(eq(new_membership))
            .returning(|_| Box::pin(async {
                Err(MockDatabaseError::constraint_violation())
            }));

        let service = setup_service(mock_repo, mock_group_service, mock_user_service);

        let result = service.create_checked_internal(create_dto).await;

        assert!(matches!(
            result,
            Err(ApiError::GroupMembershipError(GroupMembershipError::UserAlreadyInGroup))
        ));
    }

    #[tokio::test]
    async fn test_create_internal_foreign_key_violation() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_group_service = MockGroupChatServiceTrait::new();
        let mut mock_user_service = MockUserServiceTrait::new();

        let create_dto = GroupMembershipFactory::fake_create_group_membership_dto();
        let new_membership = create_dto.to_new_group_membership();

        mock_group_service.expect_find_by_id()
            .with(eq(create_dto.group_chat_id))
            .returning(|_| Box::pin(async {
                Ok(GroupChatReadDto {
                    id: 1,
                    name: "Test Group".to_string(),
                    description: "Test Description".to_string(),
                    created_by: 1,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
            }));

        mock_user_service.expect_find_by_id()
            .with(eq(create_dto.invitation_id))
            .returning(|_| Box::pin(async { Ok(UserFactory::fake_read_user_dto()) }));

        mock_repo.expect_insert()
            .with(eq(new_membership))
            .returning(|_| Box::pin(async {
                Err(MockDatabaseError::foreign_key_violation())
            }));

        let service = setup_service(mock_repo, mock_group_service, mock_user_service);

        let result = service.create_checked_internal(create_dto).await;

        assert!(matches!(
            result,
            Err(ApiError::DbError(DbError::ForeignKeyViolation(_)))
        ));
    }
}
 */