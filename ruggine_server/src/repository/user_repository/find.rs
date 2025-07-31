use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::user::User;
use crate::repository::user_repository::UserRepository;

impl UserRepository {
    pub async fn find_inner(&self, id: i32) -> Result<User, Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM \"user\" WHERE id = $1")
            .bind(id)
            .fetch_one(self.db_conn.get_pool())
            .await;
        user
    }
}

#[cfg(test)]
mod user_repository_find_tests {
    use super::*;
    use crate::entity::user::{User};
    use chrono::{Utc};
    use mockall::predicate::*;
    use crate::repository::user_repository::user_repository_trait::MockUserRepositoryTrait;
    use crate::repository::user_repository::UserRepositoryTrait;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_success() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let test_id = 1i32;
        let expected_user = User {
            id: 1,
            first_name: "Jane".to_string(),
            last_name: "Smith".to_string(),
            username: "janesmith".to_string(),
            email: "jane.smith@example.com".to_string(),
            password: "hashed_password".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_status: Default::default(), // Default user status
            user_type: Default::default(), // Default user type
            birthday: chrono::NaiveDate::from_ymd_opt(1992, 5, 15).unwrap(),
            is_online: false,
            address: "456 Oak Ave".to_string(),
            current_action: Default::default(), // Default current action
            gender: crate::entity::user::Gender::Female,
        };

        let expected_user_clone = expected_user.clone();
        mock_user_repo
            .expect_find()
            .with(eq(test_id))
            .times(1)
            .returning(move |_| {
                let user = expected_user_clone.clone();
                Box::pin(async move { Ok(user) })
            });

        // Act
        let result = mock_user_repo.find(test_id).await;

        // Assert
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.first_name, "Jane");
        assert_eq!(user.email, "jane.smith@example.com");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_not_found() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let test_id = 999i32;

        mock_user_repo
            .expect_find()
            .with(eq(test_id))
            .times(1)
            .returning(|_| Box::pin(async move { Err(Error::RowNotFound) }));

        // Act
        let result = mock_user_repo.find(test_id).await;

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::RowNotFound));
    }
}