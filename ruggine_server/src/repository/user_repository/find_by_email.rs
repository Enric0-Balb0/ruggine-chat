use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::user::User;
use crate::repository::user_repository::UserRepository;

impl UserRepository {
    pub async fn find_by_email_inner(&self, email: String) -> Result<Option<User>, Error> {
        let query = sqlx::query_as::<_, User>("SELECT * FROM \"user\" WHERE email = $1")
            .bind(email);

        let user = if let Some(mut tx_ref) = self.db_conn.get_tx_mut() {
            query.fetch_optional(&mut *tx_ref).await.unwrap_or(None)
        } else {
            query.fetch_optional(self.db_conn.get_pool()).await.unwrap_or(None)
        };

        Ok(user)
    }
}

#[cfg(test)]
mod user_repository_find_by_email_tests {
    use crate::entity::user::{User,};
    use chrono::{Utc};
    use mockall::predicate::*;
    use crate::factory::user_factory::UserFactory;
    use crate::repository::user_repository::user_repository_trait::MockUserRepositoryTrait;
    use crate::repository::user_repository::UserRepositoryTrait;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_email_success() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let test_email = UserFactory::fake_email("find_test");
        let expected_user = User {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: test_email.clone(),
            password: "hashed_password".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_status: Default::default(), // Default user status
            user_type: Default::default(), // Default user type
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            is_online: false,
            address: "123 Test St".to_string(),
            gender: Default::default(), // Default gender
        };

        let expected_user_clone = expected_user.clone();
        mock_user_repo
            .expect_find_by_email()
            .with(eq(test_email.clone()))
            .times(1)
            .returning(move |_| {
                let user = expected_user_clone.clone();
                Box::pin(async move { Ok(Some(user)) })
            });

        // Act
        let result = mock_user_repo.find_by_email(test_email).await;

        // Assert
        assert!(result.is_ok());
        let user = result.unwrap().unwrap();
        assert_eq!(user.email, "find_test@test.com");
        assert_eq!(user.first_name, "John");
        assert_eq!(user.last_name, "Doe");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_email_not_found() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let test_email = UserFactory::fake_email("nonexistent");

        mock_user_repo
            .expect_find_by_email()
            .with(eq(test_email.clone()))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(None) }));

        // Act
        let result = mock_user_repo.find_by_email(test_email).await;

        // Assert
        assert!(result.unwrap().is_none());
    }

}