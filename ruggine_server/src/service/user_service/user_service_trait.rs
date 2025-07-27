use crate::dto::user_dto::{UserReadDto, UserRegisterDto};
use crate::entity::user::User;
use crate::error::api_error::ApiError;
use async_trait::async_trait;
use mockall::automock;

#[async_trait]
#[automock]
pub trait UserServiceTrait: Send + Sync {
    async fn create_user(&self, payload: UserRegisterDto) -> Result<UserReadDto, ApiError>;
    fn verify_password(&self, user: &User, password: &str) -> bool;
}
