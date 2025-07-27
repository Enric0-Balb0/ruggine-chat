use crate::dto::user_dto::{UserReadDto, UserRegisterDto};
use crate::error::{api_error::ApiError, db_error::DbError, user_error::UserError};
use crate::service::user_service::UserService;
use sqlx::Error as SqlxError;

impl UserService {
    pub async fn create_user_internal(&self, payload: UserRegisterDto) -> Result<UserReadDto, ApiError> {
        return match self.user_repo.find_by_email(payload.email.to_owned()).await {
            Some(_) => Err(UserError::UserAlreadyExists("Username or email already taken".to_string()))?,
            None => {
                let user = self.add_user(payload).await;

                return match user {
                    Ok(user) => Ok(UserReadDto::from(user)),
                    Err(e) => match e {
                        SqlxError::Database(e) => match e.code() {
                            Some(code) => {
                                println!("{}", e.to_string());
                                if code == "23000" {
                                    Err(DbError::UniqueConstraintViolation(e.to_string()))?
                                } else {
                                    Err(DbError::SomethingWentWrong(e.to_string()))?
                                }
                            }
                            _ => Err(DbError::SomethingWentWrong(e.to_string()))?,
                        },
                        _ => {
                            println!("{}", e.to_string());
                            Err(DbError::SomethingWentWrong(e.to_string()))?
                        }
                    },
                };
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::user::{User};
    use crate::dto::user_dto::UserRegisterDto;
    use crate::repository::user_repository::MockUserRepositoryTrait;
    use chrono::{Utc};
    use std::sync::Arc;
    use mockall::predicate::*;
    use std::pin::Pin;
    use std::future::Future;

    #[tokio::test]
    async fn test_create_user_success() {
        // Arrange: Set up a mock repository with successful responses
        let mut mock_repo = MockUserRepositoryTrait::new();

        let new_user = User {
            id: 1,
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: bcrypt::hash("password", 4).unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_status: Default::default(),
            user_type: Default::default(), // Default user type
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            is_online: false,
            address: "123 Test St".to_string(),
            current_action: Default::default(),
            gender: Default::default(),
        };

        // Mock: find_by_email should return None (user not found)
        mock_repo
            .expect_find_by_email()
            .with(eq("test@example.com".to_string()))
            .returning(|_| {
                Box::pin(async { None }) as Pin<Box<dyn Future<Output = Option<User>> + Send>>
            });

        // Mock: insert should return id = 1
        mock_repo
            .expect_insert()
            .returning(|_| {
                Box::pin(async { Ok(1_i32) }) as Pin<Box<dyn Future<Output = Result<i32, sqlx::Error>> + Send>>
            });

        // Mock: find should return the created user
        let cloned_user = new_user.clone();
        mock_repo
            .expect_find()
            .with(eq(1))
            .returning(move |_| {
                let user = cloned_user.clone();
                Box::pin(async move { Ok(user) }) as Pin<Box<dyn Future<Output = Result<User, sqlx::Error>> + Send>>
            });

        let service = UserService::with_repo(Arc::new(mock_repo));

        let dto = UserRegisterDto {
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: "password".into(),
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Test St".to_string(),
            gender: crate::entity::user::Gender::Male,
        };

        // Act: Call the create_user method
        let result = service.create_user_internal(dto).await;

        // Assert: Verify the user was created successfully
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.first_name, "Test");
        assert_eq!(user.last_name, "User");
        assert_eq!(user.username, "testuser");
    }

    #[tokio::test]
    async fn test_create_user_already_exists() {
        // Arrange: Set up a mock repository where user already exists
        let mut mock_repo = MockUserRepositoryTrait::new();

        let existing_user = User {
            id: 1,
            first_name: "Existing".into(),
            last_name: "User".into(),
            username: "existinguser".into(),
            email: "test@example.com".into(),
            password: bcrypt::hash("password", 4).unwrap(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_status: Default::default(),
            user_type: Default::default(), // Default user type
            birthday: chrono::NaiveDate::from_ymd_opt(1985, 5, 10).unwrap(),
            is_online: true,
            address: "456 Oak Ave".to_string(),
            current_action: Default::default(),
            gender: crate::entity::user::Gender::Female,
        };

        // Mock: find_by_email should return Some(user) indicating user already exists
        mock_repo
            .expect_find_by_email()
            .with(eq("test@example.com".to_string()))
            .returning(move |_| {
                let user = existing_user.clone();
                Box::pin(async move { Some(user) }) as Pin<Box<dyn Future<Output = Option<User>> + Send>>
            });

        let service = UserService::with_repo(Arc::new(mock_repo));

        let dto = UserRegisterDto {
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: "password".into(),
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Test St".to_string(),
            gender: crate::entity::user::Gender::Male,
        };

        // Act: Attempt to create a user that already exists
        let result = service.create_user_internal(dto).await;

        // Assert: Should return UserAlreadyExists error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::UserError(UserError::UserAlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_create_user_database_unique_constraint_violation() {
        // Arrange: Set up a mock repository that returns a unique constraint violation
        let mut mock_repo = MockUserRepositoryTrait::new();

        // Mock: find_by_email should return None (user not found in our check)
        mock_repo
            .expect_find_by_email()
            .with(eq("test@example.com".to_string()))
            .returning(|_| {
                Box::pin(async { None }) as Pin<Box<dyn Future<Output = Option<User>> + Send>>
            });

        // Mock: insert should return a unique constraint violation error
        mock_repo
            .expect_insert()
            .returning(|_| {
                let db_error = sqlx::Error::Database(Box::new(MockDatabaseError::new("23000".to_string())));
                Box::pin(async move { Err(db_error) }) as Pin<Box<dyn Future<Output = Result<i32, sqlx::Error>> + Send>>
            });

        let service = UserService::with_repo(Arc::new(mock_repo));

        let dto = UserRegisterDto {
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: "password".into(),
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Test St".to_string(),
            gender: crate::entity::user::Gender::Male,
        };

        // Act: Attempt to create a user that violates unique constraint
        let result = service.create_user_internal(dto).await;

        // Assert: Should return UniqueConstraintViolation error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::DbError(DbError::UniqueConstraintViolation(_))));
    }

    #[tokio::test]
    async fn test_create_user_database_general_error() {
        // Arrange: Set up a mock repository that returns a general database error
        let mut mock_repo = MockUserRepositoryTrait::new();

        // Mock: find_by_email should return None (user not found)
        mock_repo
            .expect_find_by_email()
            .with(eq("test@example.com".to_string()))
            .returning(|_| {
                Box::pin(async { None }) as Pin<Box<dyn Future<Output = Option<User>> + Send>>
            });

        // Mock: insert should return a general database error
        mock_repo
            .expect_insert()
            .returning(|_| {
                let db_error = sqlx::Error::Database(Box::new(MockDatabaseError::new("42000".to_string())));
                Box::pin(async move { Err(db_error) }) as Pin<Box<dyn Future<Output = Result<i32, sqlx::Error>> + Send>>
            });

        let service = UserService::with_repo(Arc::new(mock_repo));

        let dto = UserRegisterDto {
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: "password".into(),
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Test St".to_string(),
            gender: crate::entity::user::Gender::Male,
        };

        // Act: Attempt to create a user with database error
        let result = service.create_user_internal(dto).await;

        // Assert: Should return SomethingWentWrong error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::DbError(DbError::SomethingWentWrong(_))));
    }

    #[tokio::test]
    async fn test_create_user_find_user_after_insert_fails() {
        // Arrange: Set up a mock repository where insert succeeds but find fails
        let mut mock_repo = MockUserRepositoryTrait::new();

        // Mock: find_by_email should return None (user not found)
        mock_repo
            .expect_find_by_email()
            .with(eq("test@example.com".to_string()))
            .returning(|_| {
                Box::pin(async { None }) as Pin<Box<dyn Future<Output = Option<User>> + Send>>
            });

        // Mock: insert should succeed and return id = 1
        mock_repo
            .expect_insert()
            .returning(|_| {
                Box::pin(async { Ok(1_i32) }) as Pin<Box<dyn Future<Output = Result<i32, sqlx::Error>> + Send>>
            });

        // Mock: find should fail with a database error
        mock_repo
            .expect_find()
            .with(eq(1))
            .returning(|_| {
                let error = sqlx::Error::RowNotFound;
                Box::pin(async move { Err(error) }) as Pin<Box<dyn Future<Output = Result<User, sqlx::Error>> + Send>>
            });

        let service = UserService::with_repo(Arc::new(mock_repo));

        let dto = UserRegisterDto {
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: "password".into(),
            birthday: chrono::NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Test St".to_string(),
            gender: crate::entity::user::Gender::Male,
        };

        // Act: Attempt to create a user where find fails after insert
        let result = service.create_user_internal(dto).await;

        // Assert: Should return SomethingWentWrong error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::DbError(DbError::SomethingWentWrong(_))));
    }

    // Helper struct to mock database errors
    #[derive(Debug)]
    struct MockDatabaseError {
        code: String,
        message: String,
    }

    impl MockDatabaseError {
        fn new(code: String) -> Self {
            Self {
                code: code.clone(),
                message: format!("Mock database error with code: {}", code),
            }
        }
    }

    impl std::fmt::Display for MockDatabaseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for MockDatabaseError {}

    impl sqlx::error::DatabaseError for MockDatabaseError {
        fn message(&self) -> &str {
            &self.message
        }

        fn code(&self) -> Option<std::borrow::Cow<'_, str>> {
            Some(std::borrow::Cow::Borrowed(&self.code))
        }

        fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
            self
        }
    }
}