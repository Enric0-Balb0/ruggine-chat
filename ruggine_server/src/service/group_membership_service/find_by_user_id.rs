use crate::dto::group_membership_dto::GroupMembershipReadDto;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::service::group_membership_service::GroupMembershipService;

impl GroupMembershipService {
    /// Find all group memberships for a given user ID
    /// Returns all group memberships with invitation data for the specified user
    pub async fn find_by_user_id_internal(&self, user_id: i32) -> Result<Vec<GroupMembershipReadDto>, ApiError> {
        // Retrieve all memberships for the user from the repository
        let group_memberships = self.group_membership_repo.find_by_user_id(user_id).await.map_err(|e| {
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
    use crate::repository::group_membership_repository::group_membership_repository_trait::MockGroupMembershipRepositoryTrait;
    use crate::service::group_chat_service::group_chat_service_trait::MockGroupChatServiceTrait;
    use crate::service::user_service::user_service_trait::MockUserServiceTrait;
    use crate::entity::group_membership::{MemberRole, MembershipStatus};
    use chrono::Utc;

    fn setup_service_with_mock_repo(mock_repo: MockGroupMembershipRepositoryTrait) -> GroupMembershipService {
        let mock_group_service = MockGroupChatServiceTrait::new();
        let mock_user_service = MockUserServiceTrait::new();

        GroupMembershipService::with(
            Arc::new(mock_repo),
            Arc::new(mock_group_service),
            Arc::new(mock_user_service),
        )
    }

    #[tokio::test]
    async fn test_find_by_user_id_internal_success() {
        // Arrange
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;
        let membership1 = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let membership2 = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let expected_memberships = vec![membership1.clone(), membership2.clone()];

        mock_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| {
                let memberships = expected_memberships.clone();
                Box::pin(async move { Ok(memberships) })
            });

        let service = setup_service_with_mock_repo(mock_repo);

        // Act
        let result = service.find_by_user_id_internal(user_id).await;

        // Assert
        assert!(result.is_ok());
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 2);
        
        // Verify first membership
        assert_eq!(memberships[0].id, membership1.id);
        assert_eq!(memberships[0].user_id, membership1.user_id);
        assert_eq!(memberships[0].group_chat_id, membership1.group_chat_id);
        assert_eq!(memberships[0].role, membership1.role);
        
        // Verify second membership
        assert_eq!(memberships[1].id, membership2.id);
        assert_eq!(memberships[1].user_id, membership2.user_id);
        assert_eq!(memberships[1].group_chat_id, membership2.group_chat_id);
        assert_eq!(memberships[1].role, membership2.role);
    }

    #[tokio::test]
    async fn test_find_by_user_id_internal_empty_result() {
        // Arrange
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;

        mock_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(vec![]) }));

        let service = setup_service_with_mock_repo(mock_repo);

        // Act
        let result = service.find_by_user_id_internal(user_id).await;

        // Assert
        assert!(result.is_ok());
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 0);
    }

    #[tokio::test]
    async fn test_find_by_user_id_internal_single_membership() {
        // Arrange
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;
        let membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let expected_memberships = vec![membership.clone()];

        mock_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Box::pin({
                 let value = expected_memberships.clone();
                 async move { Ok(value.clone()) }
            }));

        let service = setup_service_with_mock_repo(mock_repo);

        // Act
        let result = service.find_by_user_id_internal(user_id).await;

        // Assert
        assert!(result.is_ok());
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 1);
        assert_eq!(memberships[0].id, membership.id);
        assert_eq!(memberships[0].user_id, membership.user_id);
        assert_eq!(memberships[0].group_chat_id, membership.group_chat_id);
        assert_eq!(memberships[0].role, membership.role);
        assert_eq!(memberships[0].membership_status, membership.membership_status);
    }

    #[tokio::test]
    async fn test_find_by_user_id_internal_with_admin_role() {
        // Arrange
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;
        let mut membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        membership.role = MemberRole::Admin;
        let expected_memberships = vec![membership.clone()];

        mock_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Box::pin({
                 let value = expected_memberships.clone();
                 async move { Ok(value.clone()) }
            }));

        let service = setup_service_with_mock_repo(mock_repo);

        // Act
        let result = service.find_by_user_id_internal(user_id).await;

        // Assert
        assert!(result.is_ok());
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 1);
        assert_eq!(memberships[0].role, MemberRole::Admin);
    }

    #[tokio::test]
    async fn test_find_by_user_id_internal_with_left_membership() {
        // Arrange
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;
        let mut membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        membership.left_at = Some(Utc::now());
        membership.membership_status = MembershipStatus::Left;
        let expected_memberships = vec![membership.clone()];

        mock_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Box::pin({
                 let value = expected_memberships.clone();
                 async move { Ok(value.clone()) }
            }));

        let service = setup_service_with_mock_repo(mock_repo);

        // Act
        let result = service.find_by_user_id_internal(user_id).await;

        // Assert
        assert!(result.is_ok());
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 1);
        assert_eq!(memberships[0].membership_status, MembershipStatus::Left);
        assert!(memberships[0].left_at.is_some());
    }

    #[tokio::test]
    async fn test_find_by_user_id_internal_database_error() {
        // Arrange
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;

        mock_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Box::pin(async move { Err(sqlx::Error::PoolClosed) }));

        let service = setup_service_with_mock_repo(mock_repo);

        // Act
        let result = service.find_by_user_id_internal(user_id).await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(_)) => {
                // Expected error type
            }
            _ => panic!("Expected DbError::SomethingWentWrong"),
        }
    }

    #[tokio::test]
    async fn test_find_by_user_id_internal_mixed_membership_statuses() {
        // Arrange
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;
        
        let mut active_membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        active_membership.membership_status = MembershipStatus::Active;
        active_membership.role = MemberRole::Member;
        
        let mut left_membership = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        left_membership.membership_status = MembershipStatus::Left;
        left_membership.left_at = Some(Utc::now());
        left_membership.role = MemberRole::Admin;
        
        let expected_memberships = vec![active_membership.clone(), left_membership.clone()];

        mock_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Box::pin({
                 let value = expected_memberships.clone();
                 async move { Ok(value.clone()) }
            }));


        let service = setup_service_with_mock_repo(mock_repo);

        // Act
        let result = service.find_by_user_id_internal(user_id).await;

        // Assert
        assert!(result.is_ok());
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 2);
        
        // Find active and left memberships
        let active = memberships.iter().find(|m| m.membership_status == MembershipStatus::Active).unwrap();
        let left = memberships.iter().find(|m| m.membership_status == MembershipStatus::Left).unwrap();
        
        assert_eq!(active.role, MemberRole::Member);
        assert!(active.left_at.is_none());
        
        assert_eq!(left.role, MemberRole::Admin);
        assert!(left.left_at.is_some());
    }

    #[tokio::test]
    async fn test_find_by_user_id_internal_negative_user_id() {
        // Arrange
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = -1;

        mock_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(vec![]) }));

        let service = setup_service_with_mock_repo(mock_repo);

        // Act
        let result = service.find_by_user_id_internal(user_id).await;

        // Assert
        assert!(result.is_ok());
        let memberships = result.unwrap();
        assert_eq!(memberships.len(), 0);
    }
}
