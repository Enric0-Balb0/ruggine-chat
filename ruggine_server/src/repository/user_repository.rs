use crate::config::database::{Database, DatabaseTrait};
use crate::entity::user::{User, NewUser};
use async_trait::async_trait;
use sqlx;
use sqlx::Error;
use std::sync::Arc;
use sqlx::Error as SqlxError;

#[derive(Clone)]
pub struct UserRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait UserRepositoryTrait {
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
        let result = sqlx::query!(
            r#"
            INSERT INTO user (first_name, last_name, user_name, email, password, is_active)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            new_user.first_name,
            new_user.last_name,
            new_user.user_name,
            new_user.email,
            new_user.password,
            new_user.is_active
        )
        .execute(self.db_conn.get_pool())
        .await?;

        Ok(result.last_insert_id())
    }
}
