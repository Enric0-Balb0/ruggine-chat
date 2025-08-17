use crate::dto::group_membership_dto::GroupMembershipReadDto;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_chat_error::GroupChatError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::service::group_membership_service::GroupMembershipService;

impl GroupMembershipService {
    /// Find all group memberships for a given group ID
    /// Only authenticated users who are active members of the group can see all memberships
    /// Returns all active group memberships for the specified group
    pub async fn find_by_group_id_internal(&self, group_id: i32, auth_user_id: i32) -> Result<Vec<GroupMembershipReadDto>, ApiError> {
        // First, check if the group exists
        let _group = self.group_chat_service.find_by_id(group_id).await.map_err(|_| {
            ApiError::GroupMembershipError(GroupMembershipError::GroupNotFound)
        })?;

        // Check if the authenticated user has an active membership in this group
        let auth_membership = self.group_membership_repo
            .find_active_by_user_id_and_group_id(auth_user_id, group_id)
            .await
            .map_err(|_| {
                ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)
            })?;

        // Verify that the user's membership is active
        if auth_membership.membership_status != crate::entity::group_membership::MembershipStatus::Active {
            return Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound));
        }

        // Retrieve all active memberships for the group from the repository
        let group_memberships = self.group_membership_repo.find_by_group_id(group_id).await.map_err(|e| {
            let db_error = ApiError::DbError(DbError::SomethingWentWrong(e.to_string()));
            db_error
        })?;

        // Convert each membership to DTO
        let membership_dtos = group_memberships
            .into_iter()
            .map(GroupMembershipReadDto::from)
            .collect();

        Ok(membership_dtos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use mockall::predicate::*;
    use crate::factory::group_membership_factory::GroupMembershipFactory;
    use crate::factory::group_chat_factory::GroupChatFactory;
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use crate::entity::group_membership::{MemberRole, MembershipStatus};
    use crate::dto::group_chat_dto::GroupChatReadDto;
    use chrono::Utc;
    use sqlx::Error as SqlxError;

    fn setup_service_with_mock_repo(
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
    async fn test_find_by_group_id_internal_success() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_group_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        let group_id = 1;
        let auth_user_id = 10;
        let expected_group = GroupChatFactory::fake_group_chat_read_dto();
        let expected_auth_membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let expected_memberships = vec![
            GroupMembershipFactory::fake_group_membership_with_invitation_row(),
            GroupMembershipFactory::fake_group_membership_with_invitation_row(),
        ];

        // Mock group existence check
        mock_group_service
            .expect_find_by_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let group = GroupChatFactory::fake_group_chat_read_dto();
                Box::pin(async move { Ok(group) })
            });

        // Mock auth user membership check
        mock_repo
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                Box::pin(async move { Ok(membership) })
            });

        // Mock find all memberships for group
        mock_repo
            .expect_find_by_group_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let memberships = vec![
                    GroupMembershipFactory::fake_group_membership_with_invitation_row(),
                    GroupMembershipFactory::fake_group_membership_with_invitation_row(),
                ];
                Box::pin(async move { Ok(memberships) })
            });

        let service = setup_service_with_mock_repo(mock_repo, mock_group_service, mock_user_service);
        let result = service.find_by_group_id_internal(group_id, auth_user_id).await;

        assert!(result.is_ok());
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 2);
    }

    #[tokio::test]
    async fn test_find_by_group_id_internal_group_not_found() {
        let mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_group_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        let group_id = 999;
        let auth_user_id = 10;

        // Mock group not found
        mock_group_service
            .expect_find_by_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { 
                    Err(ApiError::GroupMembershipError(GroupMembershipError::GroupNotFound))
                })
            });

        let service = setup_service_with_mock_repo(mock_repo, mock_group_service, mock_user_service);
        let result = service.find_by_group_id_internal(group_id, auth_user_id).await;

        assert!(result.is_err());
        if let Err(ApiError::GroupMembershipError(GroupMembershipError::GroupNotFound)) = result {
            // Expected error
        } else {
            panic!("Expected GroupChatError::GroupChatNotFound, got: {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_find_by_group_id_internal_user_not_in_group() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_group_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        let group_id = 1;
        let auth_user_id = 999;

        // Mock group exists
        mock_group_service
            .expect_find_by_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let group = GroupChatFactory::fake_group_chat_read_dto();
                Box::pin(async move { Ok(group) })
            });

        // Mock user not in group
        mock_repo
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                Box::pin(async move { Err(SqlxError::RowNotFound) })
            });

        let service = setup_service_with_mock_repo(mock_repo, mock_group_service, mock_user_service);
        let result = service.find_by_group_id_internal(group_id, auth_user_id).await;

        assert!(result.is_err());
        if let Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)) = result {
            // Expected error
        } else {
            panic!("Expected GroupMembershipError::GroupMembershipNotFound, got: {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_find_by_group_id_internal_user_left_group() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_group_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        let group_id = 1;
        let auth_user_id = 10;

        // Mock group exists
        mock_group_service
            .expect_find_by_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let group = GroupChatFactory::fake_group_chat_read_dto();
                Box::pin(async move { Ok(group) })
            });

        // Mock user has left the group (inactive membership)
        mock_repo
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let mut membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                membership.membership_status = MembershipStatus::Left;
                Box::pin(async move { Ok(membership) })
            });

        let service = setup_service_with_mock_repo(mock_repo, mock_group_service, mock_user_service);
        let result = service.find_by_group_id_internal(group_id, auth_user_id).await;

        assert!(result.is_err());
        if let Err(ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound)) = result {
            // Expected error
        } else {
            panic!("Expected GroupMembershipError::GroupMembershipNotFound, got: {:?}", result);
        }
    }

    #[tokio::test]
    async fn test_find_by_group_id_internal_empty_result() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_group_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        let group_id = 1;
        let auth_user_id = 10;

        // Mock group exists
        mock_group_service
            .expect_find_by_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let group = GroupChatFactory::fake_group_chat_read_dto();
                Box::pin(async move { Ok(group) })
            });

        // Mock auth user membership check
        mock_repo
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                Box::pin(async move { Ok(membership) })
            });

        // Mock empty result for group memberships
        mock_repo
            .expect_find_by_group_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Ok(vec![]) })
            });

        let service = setup_service_with_mock_repo(mock_repo, mock_group_service, mock_user_service);
        let result = service.find_by_group_id_internal(group_id, auth_user_id).await;

        assert!(result.is_ok());
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 0);
    }

    #[tokio::test]
    async fn test_find_by_group_id_internal_database_error() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let mut mock_group_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        let group_id = 1;
        let auth_user_id = 10;

        // Mock group exists
        mock_group_service
            .expect_find_by_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                let group = GroupChatFactory::fake_group_chat_read_dto();
                Box::pin(async move { Ok(group) })
            });

        // Mock auth user membership check
        mock_repo
            .expect_find_active_by_user_id_and_group_id()
            .with(eq(auth_user_id), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                Box::pin(async move { Ok(membership) })
            });

        // Mock database error for group memberships
        mock_repo
            .expect_find_by_group_id()
            .with(eq(group_id))
            .times(1)
            .returning(move |_| {
                Box::pin(async move { Err(SqlxError::Protocol("Connection lost".to_string())) })
            });

        let service = setup_service_with_mock_repo(mock_repo, mock_group_service, mock_user_service);
        let result = service.find_by_group_id_internal(group_id, auth_user_id).await;

        assert!(result.is_err());
        if let Err(ApiError::DbError(_)) = result {
            // Expected database error
        } else {
            panic!("Expected DbError, got: {:?}", result);
        }
    }
}
