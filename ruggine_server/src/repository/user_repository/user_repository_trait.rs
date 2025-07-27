use crate::entity::user::{User, NewUser};
use async_trait::async_trait;
use sqlx::Error;
use sqlx::Error as SqlxError;
use mockall::automock;

#[async_trait]
#[automock]
pub trait UserRepositoryTrait: Send + Sync {
    async fn find_by_email(&self, email: String) -> Option<User>;
    async fn find(&self, id: i32) -> Result<User, Error>;
    async fn insert(&self, new_user: NewUser) -> Result<i32, SqlxError>;
}
