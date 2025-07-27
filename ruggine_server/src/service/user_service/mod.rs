pub mod create_user;
pub mod verify_password;
mod add_user;
pub mod user_service_trait;

use crate::config::database::Database;
use crate::repository::user_repository::{UserRepository, UserRepositoryTrait};
use std::sync::Arc;
use async_trait::async_trait;
use crate::dto::user_dto::{UserReadDto, UserRegisterDto};
use crate::entity::user::User;
use crate::error::api_error::ApiError;
pub use crate::service::user_service::user_service_trait::UserServiceTrait;

#[derive(Clone)]
pub struct UserService {
    user_repo: Arc<dyn UserRepositoryTrait>,
}

impl UserService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            user_repo: Arc::new(UserRepository::new(db_conn)),
        }
    }

    pub fn with_repo(repo: Arc<dyn UserRepositoryTrait>) -> Self {
        Self { user_repo: repo }
    }

    pub fn user_repo(&self) -> Arc<dyn UserRepositoryTrait> {
        Arc::clone(&self.user_repo)
    }
}

#[async_trait]
impl UserServiceTrait for UserService {
    async fn create_user(&self, payload: UserRegisterDto) -> Result<UserReadDto, ApiError> {
        self.create_user_internal(payload).await
    }

    fn verify_password(&self, user: &User, password: &str) -> bool {
        self.verify_password_internal(user, password)
    }
}
