use crate::dto::user_dto::{ProfileUpdateDto, UpdateOnlineDto, UserReadDto};
use crate::entity::user::UpdateUser;
use crate::error::{api_error::ApiError, db_error::DbError, user_error::UserError};
use crate::service::user_service::UserService;
use sqlx::Error as SqlxError;
use tracing::error;

impl UserService {
    pub async fn update_online_internal(&self, user_id: i32, update_online: UpdateOnlineDto) -> Result<(), ApiError> {
        let updated_user = self.user_repo.update_online(user_id, update_online.online).await;

        match updated_user {
            Ok(_) => Ok(()),
            Err(e) => match e {
                _ => {
                    error!("Update user online error: {}", e.to_string());
                    Err(DbError::SomethingWentWrong(e.to_string()))?
                }
            }
        }
    }
}