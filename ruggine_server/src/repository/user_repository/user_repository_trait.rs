use crate::entity::user::{User, NewUser, UpdateUser};
use async_trait::async_trait;
use sqlx::Error;
use sqlx::Error as SqlxError;
use mockall::automock;

#[async_trait]
#[automock]
pub trait UserRepositoryTrait: Send + Sync {
    async fn find_by_email(&self, email: String) -> Option<User>;
    async fn find_by_username(&self, username: String) -> Result<Option<User>, SqlxError>;
    async fn find(&self, id: i32) -> Result<User, Error>;
    async fn insert(&self, new_user: NewUser) -> Result<i32, SqlxError>;
    async fn update_profile(&self, user_id: i32, update_user: UpdateUser) -> Result<User, SqlxError>;
    async fn update_online(&self, user_id: i32, is_online: bool) -> Result<(), SqlxError>;
}
