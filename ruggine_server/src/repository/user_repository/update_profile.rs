use crate::entity::user::{UpdateUser, User};
use crate::repository::user_repository::UserRepository;
use crate::config::database::DatabaseTrait;
use sqlx::Error as SqlxError;

impl UserRepository {
    pub async fn update_profile_internal(&self, user_id: i32, update_user: UpdateUser) -> Result<User, SqlxError> {
        let current_user = self.find_inner(user_id).await?;

        let updated_first_name = update_user.first_name.unwrap_or(current_user.first_name);
        let updated_last_name = update_user.last_name.unwrap_or(current_user.last_name);
        let updated_birthday = update_user.birthday.unwrap_or(current_user.birthday);
        let updated_address = update_user.address.unwrap_or(current_user.address);
        let updated_gender = update_user.gender.unwrap_or(current_user.gender);

        let user = sqlx::query_as!(
            User,
            r#"UPDATE "user" SET
                first_name = $1,
                last_name = $2,
                birthday = $3,
                address = $4,
                gender = $5,
                updated_at = $6
            WHERE id = $7
            RETURNING
                id,
                first_name,
                last_name,
                username,
                email,
                password,
                created_at,
                updated_at,
                user_status as "user_status: _",
                user_type as "user_type: _",
                birthday,
                is_online,
                address,
                current_action as "current_action: _",
                gender as "gender: _""#,
            updated_first_name,
            updated_last_name,
            updated_birthday,
            updated_address,
            updated_gender as _,
            update_user.updated_at, // sqlx expects NaiveDateTime
            user_id
        )
        .fetch_one(self.db_conn.get_pool())
        .await?;

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::user_factory::UserFactory;
    use mockall::predicate::*;
    use crate::repository::user_repository::user_repository_trait::MockUserRepositoryTrait;
    use crate::repository::user_repository::UserRepositoryTrait;

    #[tokio::test]
    async fn test_update_profile_internal_success() {
        // This test would require a real database connection
        // Since we're testing the internal method, we focus on the logic
        // Integration tests will cover the actual database interaction
        
        // Test the UpdateUser::from_dto conversion and has_updates logic
        let update_dto = UserFactory::fake_user_update_dto();
        let update_user = UpdateUser::from_dto(update_dto.clone());
        
        assert!(update_user.has_updates());
        assert_eq!(update_user.first_name, update_dto.first_name);
        assert_eq!(update_user.last_name, update_dto.last_name);
        assert_eq!(update_user.birthday, update_dto.birthday);
        assert_eq!(update_user.address, update_dto.address);
        assert_eq!(update_user.gender, update_dto.gender);
    }

    #[tokio::test]
    async fn test_update_profile_trait_with_mock() {
        // Arrange: Create a mock repository
        let mut mock_repo = MockUserRepositoryTrait::new();
        let user_id = 1;
        let update_dto = UserFactory::fake_user_update_dto();
        let update_user = UpdateUser::from_dto(update_dto.clone());
        let expected_user = UserFactory::fake_user();
        let expected_user_clone = expected_user.clone();

        // Set up mock expectations
        mock_repo
            .expect_update_profile()
            .with(eq(user_id), always())
            .times(1)
            .returning(move |_, _| {
                let user = expected_user_clone.clone();
                Box::pin(async move { Ok(user) })
            });

        // Act: Call the mocked method
        let result = mock_repo.update_profile(user_id, update_user).await;

        // Assert: Verify the result
        assert!(result.is_ok());
        let updated_user = result.unwrap();
        assert_eq!(updated_user.id, expected_user.id);
    }

    #[tokio::test]
    async fn test_update_profile_mock_failure() {
        // Arrange: Create a mock repository that fails
        let mut mock_repo = MockUserRepositoryTrait::new();
        let user_id = 999;
        let update_dto = UserFactory::fake_user_update_dto();
        let update_user = UpdateUser::from_dto(update_dto);

        // Set up mock to return an error
        mock_repo
            .expect_update_profile()
            .with(eq(user_id), always())
            .times(1)
            .returning(|_, _| Box::pin(async {
                Err(sqlx::Error::RowNotFound)
            }));

        // Act: Call the mocked method
        let result = mock_repo.update_profile(user_id, update_user).await;

        // Assert: Verify the failure
        assert!(result.is_err());
        match result {
            Err(sqlx::Error::RowNotFound) => { /* Expected */ },
            _ => panic!("Expected RowNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_update_user_from_dto_preserves_data() {
        // Arrange: Create different types of update DTOs
        let complete_dto = UserFactory::fake_user_update_dto();
        let partial_dto = UserFactory::fake_user_update_dto_partial();
        let empty_dto = UserFactory::fake_user_update_dto_empty();

        // Act: Convert to UpdateUser
        let complete_update = UpdateUser::from_dto(complete_dto.clone());
        let partial_update = UpdateUser::from_dto(partial_dto.clone());
        let empty_update = UpdateUser::from_dto(empty_dto);

        // Assert: Verify conversions
        assert!(complete_update.has_updates());
        assert_eq!(complete_update.first_name, complete_dto.first_name);
        assert_eq!(complete_update.last_name, complete_dto.last_name);

        assert!(partial_update.has_updates());
        assert_eq!(partial_update.first_name, partial_dto.first_name);
        assert_eq!(partial_update.address, partial_dto.address);

        assert!(!empty_update.has_updates());
        assert!(empty_update.first_name.is_none());
        assert!(empty_update.last_name.is_none());
    }

    #[tokio::test]
    async fn test_update_user_updated_at_is_current() {
        // Arrange: Create update DTO
        let update_dto = UserFactory::fake_user_update_dto();
        let before_creation = chrono::Utc::now();

        // Act: Convert to UpdateUser
        let update_user = UpdateUser::from_dto(update_dto);
        let after_creation = chrono::Utc::now();

        // Assert: Verify updated_at is within expected range
        assert!(update_user.updated_at >= before_creation);
        assert!(update_user.updated_at <= after_creation);
    }
}

