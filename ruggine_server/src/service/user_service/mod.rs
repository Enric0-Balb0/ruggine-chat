mod create_user;
mod verify_password;
mod add_user;
pub mod user_service;
pub mod user_service_trait;

use async_trait::async_trait;
use crate::dto::user_dto::{UserReadDto, UserRegisterDto};
use crate::entity::user::User;
use crate::error::api_error::ApiError;
pub use crate::service::user_service::user_service::UserService;
pub use crate::service::user_service::user_service_trait::UserServiceTrait;



#[async_trait]
impl UserServiceTrait for UserService {
    async fn create_user(&self, payload: UserRegisterDto) -> Result<UserReadDto, ApiError> {
        self.create_user_internal(payload).await
    }

    fn verify_password(&self, user: &User, password: &str) -> bool {
        self.verify_password_internal(user, password)
    }
}
