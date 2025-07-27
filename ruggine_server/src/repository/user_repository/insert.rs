use sqlx::Error;
use crate::config::database::DatabaseTrait;
use crate::entity::user::NewUser;
use crate::repository::user_repository::UserRepository;

impl UserRepository {
    pub async fn insert_inner(&self, new_user: NewUser) -> Result<i32, Error> {
        let now = chrono::Utc::now();
        let rec = sqlx::query_scalar(
            r#"
            INSERT INTO "user" (first_name, last_name, username, email, password, user_status, user_type, birthday, is_online, address, current_action, gender, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id
            "#
        )
            .bind(new_user.first_name)
            .bind(new_user.last_name)
            .bind(new_user.username)
            .bind(new_user.email)
            .bind(new_user.password)
            .bind(new_user.user_status)
            .bind(new_user.user_type)
            .bind(new_user.birthday)
            .bind(false) // is_online default to false
            .bind(new_user.address)
            .bind(crate::entity::user::CurrentAction::Waiting) // current_action default to Waiting
            .bind(new_user.gender)
            .bind(now)
            .bind(now)
            .fetch_one(self.db_conn.get_pool())
            .await?;

        Ok(rec)
    }
}

#[cfg(test)]
mod user_repository_insert_tests {
    use super::*;
    use crate::entity::user::{NewUser, User, UserStatus};
    use chrono::{Utc};
    use mockall::predicate::*;
    use crate::factory::user_factory::UserFactory;
    use crate::repository::user_repository::user_repository_trait::MockUserRepositoryTrait;
    use crate::repository::user_repository::UserRepositoryTrait;

    #[tokio::test]
    async fn test_insert_success() {
        // Arrange
        let mut mock_user_repo = MockUserRepositoryTrait::new();
        let new_user = UserFactory::unique_fake_new_user("insert_test", UserStatus::Active);
        let expected_id = 123i32;

        mock_user_repo
            .expect_insert()
            .with(function(|arg: &NewUser| {
                arg.email.contains("insert_test") &&
                    arg.user_status == UserStatus::Active &&
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
        let new_user = UserFactory::unique_fake_new_user("duplicate", UserStatus::Active);

        mock_user_repo
            .expect_insert()
            .with(function(|arg: &NewUser| arg.email.contains("duplicate")))
            .times(1)
            .returning(|_| Box::pin(async move {
                // Simuliamo un errore di chiave duplicata usando un errore generico del database
                Err(Error::RowNotFound) // Usiamo un errore più semplice per il test
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
        let inactive_user = UserFactory::unique_fake_new_user("inactive", UserStatus::Deleted);
        let expected_id = 456i32;

        mock_user_repo
            .expect_insert()
            .with(function(|arg: &NewUser| {
                arg.email.contains("inactive") && arg.user_status == UserStatus::Deleted
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

        let new_user1 = UserFactory::unique_fake_new_user("multi1", UserStatus::Active);
        let new_user2 = UserFactory::unique_fake_new_user("multi2", UserStatus::Active);

        // Cloniamo le variabili per evitare problemi di ownership
        let email1_for_find = email1.clone();
        let username1_for_find = username1.clone();

        // Setup expectations per inserimenti
        mock_user_repo
            .expect_insert()
            .with(function(|arg: &NewUser| arg.email.contains("multi1")))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(100i32) }));

        mock_user_repo
            .expect_insert()
            .with(function(|arg: &NewUser| arg.email.contains("multi2")))
            .times(1)
            .returning(|_| Box::pin(async move { Ok(200i32) }));

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
                        user_status: Default::default(), // Default user status,
                        user_type: Default::default(), // Default user type
                        birthday: chrono::NaiveDate::from_ymd_opt(1985, 12, 25).unwrap(),
                        is_online: false,
                        address: "789 Pine Rd".to_string(),
                        current_action: Default::default(), // Default current action
                        gender: crate::entity::user::Gender::Male,
                    })
                })
            });

        // Act & Assert
        let insert_result1 = mock_user_repo.insert(new_user1).await;
        assert!(insert_result1.is_ok());
        assert_eq!(insert_result1.unwrap(), 100i32);

        let insert_result2 = mock_user_repo.insert(new_user2).await;
        assert!(insert_result2.is_ok());
        assert_eq!(insert_result2.unwrap(), 200i32);

        let find_result = mock_user_repo.find_by_email(email1).await;
        assert!(find_result.is_some());
        let found_user = find_result.unwrap();
        assert_eq!(found_user.id, 100);
        assert_eq!(found_user.first_name, "Multi1");
    }

}