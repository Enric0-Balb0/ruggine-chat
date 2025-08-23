use crate::config::database::DatabaseTrait;
use crate::entity::user::User;
use crate::repository::user_repository::UserRepository;

impl UserRepository {
    pub async fn find_by_username_inner(&self, username: String) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM \"user\" WHERE username = $1")
            .bind(username)
            .fetch_optional(self.db_conn.get_pool())
            .await;
        user
    }
}

#[cfg(test)]
mod user_repository_find_by_username_tests {
    use crate::entity::user::User;
    use chrono::Utc;
    use mockall::predicate::*;
    use crate::factory::user_factory::UserFactory;
    use crate::repository::user_repository::user_repository_trait::MockUserRepositoryTrait;
    use crate::repository::user_repository::UserRepositoryTrait;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_success() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let test_username = UserFactory::fake_username("find_test");
        let expected_user = User {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: test_username.clone(),
            email: "findtest@test.com".to_string(),
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
            .expect_find_by_username()
            .with(eq(test_username.clone()))
            .times(1)
            .returning(move |_| {
                let user = expected_user_clone.clone();
                Box::pin(async move { Ok(Some(user)) })
            });

        // Act
        let result = mock_user_repo.find_by_username(test_username.clone()).await;

        // Assert
        let optional_user = result.unwrap();
        assert!(optional_user.is_some());
        let user = optional_user.unwrap();
        assert_eq!(user.username, test_username);
        assert_eq!(user.first_name, "John");
        assert_eq!(user.last_name, "Doe");
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_not_found() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let test_username = UserFactory::fake_username("nonexistent");

        mock_user_repo
            .expect_find_by_username()
            .with(eq(test_username.clone()))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(None) }));

        // Act
        let result = mock_user_repo.find_by_username(test_username).await;

        // Assert
        assert!(result.unwrap().is_none());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_empty_string() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let empty_username = "".to_string();

        mock_user_repo
            .expect_find_by_username()
            .with(eq(empty_username.clone()))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(None) }));

        // Act
        let result = mock_user_repo.find_by_username(empty_username).await;

        // Assert
        assert!(result.unwrap().is_none());
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_username_case_sensitive() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let lowercase_username = UserFactory::fake_username("testuser");
        let uppercase_username = "TEST_TESTUSER".to_string();
        
        let expected_user = User {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: lowercase_username.clone(),
            email: "test@test.com".to_string(),
            password: "hashed_password".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_status: Default::default(),
            user_type: Default::default(),
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            is_online: false,
            address: "123 Test St".to_string(),
            gender: Default::default(),
        };

        let expected_user_clone = expected_user.clone();
        mock_user_repo
            .expect_find_by_username()
            .with(eq(lowercase_username.clone()))
            .times(1)
            .returning(move |_| {
                let user = expected_user_clone.clone();
                Box::pin(async move { Ok(Some(user)) })
            });

        mock_user_repo
            .expect_find_by_username()
            .with(eq(uppercase_username.clone()))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(None) }));

        // Act & Assert
        let result_lowercase = mock_user_repo.find_by_username(lowercase_username).await;
        assert!(result_lowercase.unwrap().is_some());

        let result_uppercase = mock_user_repo.find_by_username(uppercase_username).await;
        assert!(result_uppercase.unwrap().is_none());
    }
}
