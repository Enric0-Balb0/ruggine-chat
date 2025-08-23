pub mod user_repository;
pub mod user_repository_trait;
mod find_by_email;
mod find;
mod insert;
mod update_profile;
mod find_by_username;

use async_trait::async_trait;
use sqlx::Error;
pub use user_repository::UserRepository;
pub use user_repository_trait::UserRepositoryTrait;
use crate::entity::user::{NewUser, User, UpdateUser};

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    async fn find_by_email(&self, email: String) -> Option<User> {
        self.find_by_email_inner(email).await
    }

    async fn find(&self, id: i32) -> Result<User, Error> {
        self.find_inner(id).await
    }

    async fn insert(&self, new_user: NewUser) -> Result<i32, Error> {
        self.insert_inner(new_user).await
    }

    async fn update_profile(&self, user_id: i32, update_user: UpdateUser) -> Result<User, Error> {
        self.update_profile_internal(user_id, update_user).await
    }

    async fn find_by_username(&self, username: String) -> Result<Option<User>, Error> {
        self.find_by_username_inner(username).await
    }
}