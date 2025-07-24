use crate::config::database::{Database};
use crate::dto::user_dto::{UserReadDto, UserRegisterDto};
use crate::entity::user::{NewUser, User};
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::user_error::UserError;
use crate::repository::user_repository::{UserRepository, UserRepositoryTrait};
use axum::async_trait;
use sqlx::Error as SqlxError;
use std::sync::Arc;
use mockall::automock;

#[derive(Clone)]
pub struct UserService {
    user_repo: Arc<dyn UserRepositoryTrait>,
    // db_conn: Arc<Database>,
}

#[async_trait]
#[automock]
pub trait UserServiceTrait: Send + Sync {
    async fn create_user(&self, payload: UserRegisterDto) -> Result<UserReadDto, ApiError>;
    fn verify_password(&self, user: &User, password: &str) -> bool;
}

impl UserService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            user_repo: Arc::new(UserRepository::new(db_conn)),
            // db_conn: Arc::clone(db_conn),
        }
    }

    /// Creates a new UserService with a specific repository.
    pub fn with_repo(repo: Arc<dyn UserRepositoryTrait>) -> Self {
        Self {
            user_repo: repo,
        }
    }

    async fn add_user(&self, payload: UserRegisterDto) -> Result<User, SqlxError> {
        let hashed_password = bcrypt::hash(payload.password, 4).unwrap();

        let new_user = NewUser {
            first_name: payload.first_name,
            last_name: payload.last_name,
            username: payload.username,
            email: payload.email,
            password: hashed_password,
            is_active: 1,
        };

        let user_id = self.user_repo.insert(new_user).await?;
        let user = self.user_repo.find(user_id).await?;

        Ok(user)
    }

}

#[async_trait]
impl UserServiceTrait for UserService {
    async fn create_user(&self, payload: UserRegisterDto) -> Result<UserReadDto, ApiError> {
        return match self.user_repo.find_by_email(payload.email.to_owned()).await {
            Some(_) => Err(UserError::UserAlreadyExists)?,
            None => {
                let user = self.add_user(payload).await;

                return match user {
                    Ok(user) => Ok(UserReadDto::from(user)),
                    Err(e) => match e {
                        SqlxError::Database(e) => match e.code() {
                            Some(code) => {
                                if code == "23000" {
                                    Err(DbError::UniqueConstraintViolation(e.to_string()))?
                                } else {
                                    Err(DbError::SomethingWentWrong(e.to_string()))?
                                }
                            }
                            _ => Err(DbError::SomethingWentWrong(e.to_string()))?,
                        },
                        _ => Err(DbError::SomethingWentWrong(e.to_string()))?,
                    },
                };
            }
        };
    }

    
    fn verify_password(&self, user: &User, password: &str) -> bool {
        bcrypt::verify(password, &user.password).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::user::User;
    use crate::dto::user_dto::UserRegisterDto;
    use crate::repository::user_repository::MockUserRepositoryTrait;
    use chrono::Utc;
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
            updated_at: None,
            is_active: 1,
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
                Box::pin(async { Ok(1_u64) }) as Pin<Box<dyn Future<Output = Result<u64, sqlx::Error>> + Send>>
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
        };

        // Act: Call the create_user method
        let result = service.create_user(dto).await;

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
            updated_at: None,
            is_active: 1,
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
        };

        // Act: Attempt to create a user that already exists
        let result = service.create_user(dto).await;

        // Assert: Should return UserAlreadyExists error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::UserError(UserError::UserAlreadyExists)));
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
                Box::pin(async move { Err(db_error) }) as Pin<Box<dyn Future<Output = Result<u64, sqlx::Error>> + Send>>
            });

        let service = UserService::with_repo(Arc::new(mock_repo));

        let dto = UserRegisterDto {
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: "password".into(),
        };

        // Act: Attempt to create a user that violates unique constraint
        let result = service.create_user(dto).await;

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
                Box::pin(async move { Err(db_error) }) as Pin<Box<dyn Future<Output = Result<u64, sqlx::Error>> + Send>>
            });

        let service = UserService::with_repo(Arc::new(mock_repo));

        let dto = UserRegisterDto {
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: "password".into(),
        };

        // Act: Attempt to create a user with database error
        let result = service.create_user(dto).await;

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
                Box::pin(async { Ok(1_u64) }) as Pin<Box<dyn Future<Output = Result<u64, sqlx::Error>> + Send>>
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
        };

        // Act: Attempt to create a user where find fails after insert
        let result = service.create_user(dto).await;

        // Assert: Should return SomethingWentWrong error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(matches!(error, ApiError::DbError(DbError::SomethingWentWrong(_))));
    }

    #[test]
    fn test_verify_password_correct() {
        // Arrange: Create a user with a known password
        let password = "testpassword123";
        let hashed_password = bcrypt::hash(password, 4).unwrap();
        
        let user = User {
            id: 1,
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: hashed_password,
            created_at: Utc::now(),
            updated_at: None,
            is_active: 1,
        };

        let mock_repo = MockUserRepositoryTrait::new();
        let service = UserService::with_repo(Arc::new(mock_repo));

        // Act: Verify the correct password
        let result = service.verify_password(&user, password);

        // Assert: Should return true for correct password
        assert!(result);
    }

    #[test]
    fn test_verify_password_incorrect() {
        // Arrange: Create a user with a known password
        let correct_password = "testpassword123";
        let incorrect_password = "wrongpassword";
        let hashed_password = bcrypt::hash(correct_password, 4).unwrap();
        
        let user = User {
            id: 1,
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: hashed_password,
            created_at: Utc::now(),
            updated_at: None,
            is_active: 1,
        };

        let mock_repo = MockUserRepositoryTrait::new();
        let service = UserService::with_repo(Arc::new(mock_repo));

        // Act: Verify an incorrect password
        let result = service.verify_password(&user, incorrect_password);

        // Assert: Should return false for incorrect password
        assert!(!result);
    }

    #[test]
    fn test_verify_password_empty_password() {
        // Arrange: Create a user with a known password
        let correct_password = "testpassword123";
        let hashed_password = bcrypt::hash(correct_password, 4).unwrap();
        
        let user = User {
            id: 1,
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: hashed_password,
            created_at: Utc::now(),
            updated_at: None,
            is_active: 1,
        };

        let mock_repo = MockUserRepositoryTrait::new();
        let service = UserService::with_repo(Arc::new(mock_repo));

        // Act: Verify an empty password
        let result = service.verify_password(&user, "");

        // Assert: Should return false for empty password
        assert!(!result);
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
