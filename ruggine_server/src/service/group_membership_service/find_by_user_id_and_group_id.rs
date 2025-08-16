use crate::dto::group_membership_dto::GroupMembershipReadDto;
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::group_membership_error::GroupMembershipError;
use crate::service::group_membership_service::GroupMembershipService;

impl GroupMembershipService {
    /// Find group membership for a specific user in a specific group
    /// Returns the membership record for the user in the specified group
    pub async fn find_by_user_id_and_group_id_internal(&self, user_id: i32, group_id: i32) -> Result<GroupMembershipReadDto, ApiError> {
        // Retrieve the membership from the repository
        let group_membership = self.group_membership_repo.find_by_user_id_and_group_id(user_id, group_id).await.map_err(|e| {
            match e {
                sqlx::Error::RowNotFound => ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound),
                _ => {
                    let db_error = ApiError::DbError(DbError::SomethingWentWrong(e.to_string()));
                    db_error
                }
            }
        })?;

        // Convert to DTO
        let membership_dto = GroupMembershipReadDto::from(group_membership);

        Ok(membership_dto)
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
    use crate::model::group_membership_model::GroupMembershipWithInvitationRow;
    use chrono::Utc;
    use sqlx::Error as SqlxError;

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
    async fn test_find_by_user_id_and_group_id_success() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;
        let group_id = 10;

        let expected_row = GroupMembershipFactory::fake_group_membership_with_invitation_row();
        let expected_dto = GroupMembershipReadDto::from(expected_row.clone());

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let mut result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                result.user_id = user_id;
                result.group_chat_id = group_id;
                result.membership_status = MembershipStatus::Active;
                Box::pin(async move { Ok(result) })
            });

        let service = setup_service_with_mock_repo(mock_repo);
        let result = service.find_by_user_id_and_group_id_internal(user_id, group_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.user_id, user_id);
        assert_eq!(found.group_chat_id, group_id);
        assert_eq!(found.membership_status, MembershipStatus::Active);
    }

    #[tokio::test]
    async fn test_find_by_user_id_and_group_id_not_found() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 999;
        let group_id = 888;

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id))
            .times(1)
            .returning(|_, _| {
                Box::pin(async move { Err(SqlxError::RowNotFound) })
            });

        let service = setup_service_with_mock_repo(mock_repo);
        let result = service.find_by_user_id_and_group_id_internal(user_id, group_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            _ => panic!("Expected GroupMembershipNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_find_by_user_id_and_group_id_admin_membership() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 5;
        let group_id = 15;

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let mut result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                result.user_id = user_id;
                result.group_chat_id = group_id;
                result.role = MemberRole::Admin;
                result.membership_status = MembershipStatus::Active;
                result.joined_at = Utc::now();
                Box::pin(async move { Ok(result) })
            });

        let service = setup_service_with_mock_repo(mock_repo);
        let result = service.find_by_user_id_and_group_id_internal(user_id, group_id).await;

        assert!(result.is_ok());
        let found = result.unwrap();
        assert_eq!(found.user_id, user_id);
        assert_eq!(found.group_chat_id, group_id);
        assert_eq!(found.role, MemberRole::Admin);
        assert_eq!(found.membership_status, MembershipStatus::Active);
    }

    #[tokio::test]
    async fn test_find_by_user_id_and_group_id_database_error() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;
        let group_id = 1;

        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id))
            .times(1)
            .returning(|_, _| {
                Box::pin(async move {
                    Err(SqlxError::PoolClosed)
                })
            });

        let service = setup_service_with_mock_repo(mock_repo);
        let result = service.find_by_user_id_and_group_id_internal(user_id, group_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ApiError::DbError(DbError::SomethingWentWrong(_)) => {
                // Expected error
            }
            _ => panic!("Expected SomethingWentWrong error"),
        }
    }

    #[tokio::test]
    async fn test_find_by_user_id_and_group_id_different_users_same_group() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let group_id = 20;
        let user_id_1 = 10;
        let user_id_2 = 11;

        // First user (Admin)
        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id_1), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let mut result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                result.user_id = user_id_1;
                result.group_chat_id = group_id;
                result.role = MemberRole::Admin;
                Box::pin(async move { Ok(result) })
            });

        // Second user (Member)
        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id_2), eq(group_id))
            .times(1)
            .returning(move |_, _| {
                let mut result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                result.user_id = user_id_2;
                result.group_chat_id = group_id;
                result.role = MemberRole::Member;
                Box::pin(async move { Ok(result) })
            });

        let service = setup_service_with_mock_repo(mock_repo);
        
        let result1 = service.find_by_user_id_and_group_id_internal(user_id_1, group_id).await;
        let result2 = service.find_by_user_id_and_group_id_internal(user_id_2, group_id).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        let found1 = result1.unwrap();
        let found2 = result2.unwrap();
        
        assert_eq!(found1.user_id, user_id_1);
        assert_eq!(found1.role, MemberRole::Admin);
        assert_eq!(found2.user_id, user_id_2);
        assert_eq!(found2.role, MemberRole::Member);
        assert_eq!(found1.group_chat_id, found2.group_chat_id);
    }

    #[tokio::test]
    async fn test_find_by_user_id_and_group_id_multiple_calls_same_user() {
        let mut mock_repo = MockGroupMembershipRepositoryTrait::new();
        let user_id = 1;
        let group_id_1 = 10;
        let group_id_2 = 20;

        // First group
        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id_1))
            .times(1)
            .returning(move |_, _| {
                let mut result = GroupMembershipFactory::fake_group_membership_with_invitation_row();
                result.user_id = user_id;
                result.group_chat_id = group_id_1;
                result.role = MemberRole::Member;
                Box::pin(async move { Ok(result) })
            });

        // Second group - not found
        mock_repo
            .expect_find_by_user_id_and_group_id()
            .with(eq(user_id), eq(group_id_2))
            .times(1)
            .returning(|_, _| {
                Box::pin(async move { Err(SqlxError::RowNotFound) })
            });

        let service = setup_service_with_mock_repo(mock_repo);
        
        let result1 = service.find_by_user_id_and_group_id_internal(user_id, group_id_1).await;
        let result2 = service.find_by_user_id_and_group_id_internal(user_id, group_id_2).await;

        assert!(result1.is_ok());
        assert!(result2.is_err());
        
        let found1 = result1.unwrap();
        assert_eq!(found1.user_id, user_id);
        assert_eq!(found1.group_chat_id, group_id_1);
        assert_eq!(found1.role, MemberRole::Member);

        match result2.unwrap_err() {
            ApiError::GroupMembershipError(GroupMembershipError::GroupMembershipNotFound) => {
                // Expected error
            }
            _ => panic!("Expected GroupMembershipNotFound error"),
        }
    }
}
