use crate::config::database::{Database, DatabaseTrait};
use crate::entity::user::{User, NewUser};
use async_trait::async_trait;
use sqlx;
use sqlx::Error;
use std::sync::Arc;
use sqlx::Error as SqlxError;
use mockall::automock;

#[derive(Clone)]
pub struct UserRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
#[automock]
pub trait UserRepositoryTrait: Send + Sync {
    async fn find_by_email(&self, email: String) -> Option<User>;
    async fn find(&self, id: u64) -> Result<User, Error>;
    async fn insert(&self, new_user: NewUser) -> Result<u64, SqlxError>;
}

impl UserRepository {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    pub async fn delete_by_email(&self, email: String) -> Result<u64, SqlxError> {
        let result = sqlx::query!(
            "DELETE FROM user WHERE email = ?",
            email
        )
        .execute(self.db_conn.get_pool())
        .await?;

        Ok(result.rows_affected())
    }
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    async fn find_by_email(&self, email: String) -> Option<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM user WHERE email = ?")
            .bind(email)
            .fetch_optional(self.db_conn.get_pool())
            .await
            .unwrap_or(None);
        user
    }

    async fn find(&self, id: u64) -> Result<User, Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM user WHERE id = ?")
            .bind(id)
            .fetch_one(self.db_conn.get_pool())
            .await;
        user
    }

    async fn insert(&self, new_user: NewUser) -> Result<u64, SqlxError> {
        let now = chrono::Utc::now();
        let result = sqlx::query!(
            r#"
            INSERT INTO user (first_name, last_name, username, email, password, is_active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            new_user.first_name,
            new_user.last_name,
            new_user.username,
            new_user.email,
            new_user.password,
            new_user.is_active,
            now,
            now
        )
        .execute(self.db_conn.get_pool())
        .await?;

        Ok(result.last_insert_id())
    }
}

#[cfg(test)]
mod user_repository_unit_tests {
    use super::*;
    use crate::entity::user::{User, NewUser};
    use chrono::{Utc};
    use mockall::predicate::*;
    use crate::factory::user_factory::UserFactory;

    // Test per find_by_email con mock
    #[tokio::test]
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
            is_active: 1,
        };

        let expected_user_clone = expected_user.clone();
        mock_user_repo
            .expect_find_by_email()
            .with(eq(test_email.clone()))
            .times(1)
            .returning(move |_| {
                let user = expected_user_clone.clone();
                Box::pin(async move { Some(user) })
            });

        // Act
        let result = mock_user_repo.find_by_email(test_email).await;

        // Assert
        assert!(result.is_some());
        let user = result.unwrap();
        assert_eq!(user.email, "find_test@test.com");
        assert_eq!(user.first_name, "John");
        assert_eq!(user.last_name, "Doe");
    }

    #[tokio::test]
    async fn test_find_by_email_not_found() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let test_email = UserFactory::fake_email("nonexistent");

        mock_user_repo
            .expect_find_by_email()
            .with(eq(test_email.clone()))
            .times(1)
            .returning(|_| Box::pin(async move { None }));

        // Act
        let result = mock_user_repo.find_by_email(test_email).await;

        // Assert
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_by_id_success() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let test_id = 1u64;
        let expected_user = User {
            id: 1,
            first_name: "Jane".to_string(),
            last_name: "Smith".to_string(),
            username: "janesmith".to_string(),
            email: "jane.smith@example.com".to_string(),
            password: "hashed_password".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            is_active: 1,
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

    #[tokio::test]
    async fn test_find_by_id_not_found() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let test_id = 999u64;

        mock_user_repo
            .expect_find()
            .with(eq(test_id))
            .times(1)
            .returning(|_| Box::pin(async move { Err(SqlxError::RowNotFound) }));

        // Act
        let result = mock_user_repo.find(test_id).await;

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SqlxError::RowNotFound));
    }

    #[tokio::test]
    async fn test_insert_success() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let new_user = UserFactory::unique_fake_new_user("insert_test", 1);
        let expected_id = 123u64;

        mock_user_repo
            .expect_insert()
            .with(function(|arg: &NewUser| {
                arg.email.contains("insert_test") && 
                arg.is_active == 1 &&
                arg.password == "hashed_password"
            }))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(expected_id) }));

        // Act
        let result = mock_user_repo.insert(new_user).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_id);
    }

    #[tokio::test]
    async fn test_insert_with_duplicate_email() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let new_user = UserFactory::unique_fake_new_user("duplicate", 1);

        mock_user_repo
            .expect_insert()
            .with(function(|arg: &NewUser| arg.email.contains("duplicate")))
            .times(1)
            .returning(|_| Box::pin(async move { 
                // Simuliamo un errore di chiave duplicata usando un errore generico del database
                Err(SqlxError::RowNotFound) // Usiamo un errore più semplice per il test
            }));

        // Act
        let result = mock_user_repo.insert(new_user).await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_insert_inactive_user() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let inactive_user = UserFactory::unique_fake_new_user("inactive", 0);
        let expected_id = 456u64;

        mock_user_repo
            .expect_insert()
            .with(function(|arg: &NewUser| {
                arg.email.contains("inactive") && arg.is_active == 0
            }))
            .times(1)
            .returning(move |_| Box::pin(async move { Ok(expected_id) }));

        // Act
        let result = mock_user_repo.insert(inactive_user).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_id);
    }

    #[tokio::test]
    async fn test_multiple_operations_with_factory() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        
        // Test dati per operazioni multiple
        let (email1, username1, _) = UserFactory::get_unique_user_information("multi1");

        let new_user1 = UserFactory::unique_fake_new_user("multi1", 1);
        let new_user2 = UserFactory::unique_fake_new_user("multi2", 1);
        
        // Cloniamo le variabili per evitare problemi di ownership
        let email1_for_find = email1.clone();
        let username1_for_find = username1.clone();
        
        // Setup expectations per inserimenti
        mock_user_repo
            .expect_insert()
            .with(function(|arg: &NewUser| arg.email.contains("multi1")))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(100u64) }));
            
        mock_user_repo
            .expect_insert()
            .with(function(|arg: &NewUser| arg.email.contains("multi2")))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(200u64) }));

        // Setup expectations per find_by_email
        mock_user_repo
            .expect_find_by_email()
            .with(function(|email: &String| email.contains("multi1")))
            .times(1)
            .returning(move |_| {
                let username1_clone = username1_for_find.clone();
                let email1_clone = email1_for_find.clone();
                Box::pin(async move {
                    Some(User {
                        id: 100,
                        first_name: "Multi1".to_string(),
                        last_name: "Test".to_string(),
                        username: username1_clone,
                        email: email1_clone,
                        password: "hashed_password".to_string(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        is_active: 1,
                    })
                })
            });

        // Act & Assert
        let insert_result1 = mock_user_repo.insert(new_user1).await;
        assert!(insert_result1.is_ok());
        assert_eq!(insert_result1.unwrap(), 100u64);

        let insert_result2 = mock_user_repo.insert(new_user2).await;
        assert!(insert_result2.is_ok());
        assert_eq!(insert_result2.unwrap(), 200u64);

        let find_result = mock_user_repo.find_by_email(email1).await;
        assert!(find_result.is_some());
        let found_user = find_result.unwrap();
        assert_eq!(found_user.id, 100);
        assert_eq!(found_user.first_name, "Multi1");
    }
}
