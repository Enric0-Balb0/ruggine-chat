mod create_user_service;
mod verify_password_service;
mod add_user;
mod update_profile_service;
mod find_by_id;
pub mod user_service;
pub mod user_service_trait;


use async_trait::async_trait;
use crate::dto::user_dto::{ProfileUpdateDto, UserReadDto, UserRegisterDto};
use crate::entity::user::{User, UpdateUser};
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

    async fn update_user_profile(&self, user_id: i32, update_user: ProfileUpdateDto) -> Result<UserReadDto, ApiError> {
        self.update_user_profile_internal(user_id, update_user).await
    }

    async fn find_by_id(&self, id: i32) -> Result<UserReadDto, ApiError> {
        self.find_by_id_internal(id).await
    }
}
