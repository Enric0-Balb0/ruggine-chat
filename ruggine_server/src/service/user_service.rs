use crate::config::database::{Database};
use crate::dto::user_dto::{UserReadDto, UserRegisterDto};
use crate::entity::user::{NewUser, User};
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::error::user_error::UserError;
use crate::repository::user_repository::{UserRepository, UserRepositoryTrait};
use axum::async_trait;
use sqlx::Error as SqlxError;
use std::sync::Arc;
use mockall::automock;

#[derive(Clone)]
pub struct UserService {
    user_repo: UserRepository,
    // db_conn: Arc<Database>,
}

#[async_trait]
#[automock]
pub trait UserServiceTrait: Send + Sync {
    async fn create_user(&self, payload: UserRegisterDto) -> Result<UserReadDto, ApiError>;
    fn verify_password(&self, user: &User, password: &str) -> bool;
}

impl UserService {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            user_repo: UserRepository::new(db_conn),
            // db_conn: Arc::clone(db_conn),
        }
    }

    async fn add_user(&self, payload: UserRegisterDto) -> Result<User, SqlxError> {
        let hashed_password = bcrypt::hash(payload.password, 4).unwrap();

        let new_user = NewUser {
            first_name: payload.first_name,
            last_name: payload.last_name,
            user_name: payload.user_name,
            email: payload.email,
            password: hashed_password,
            is_active: 1,
        };

        let user_id = self.user_repo.insert(new_user).await?;
        let user = self.user_repo.find(user_id).await?;

        Ok(user)
    }

}

#[async_trait]
impl UserServiceTrait for UserService {
    async fn create_user(&self, payload: UserRegisterDto) -> Result<UserReadDto, ApiError> {
        return match self.user_repo.find_by_email(payload.email.to_owned()).await {
            Some(_) => Err(UserError::UserAlreadyExists)?,
            None => {
                let user = self.add_user(payload).await;

                return match user {
                    Ok(user) => Ok(UserReadDto::from(user)),
                    Err(e) => match e {
                        SqlxError::Database(e) => match e.code() {
                            Some(code) => {
                                if code == "23000" {
                                    Err(DbError::UniqueConstraintViolation(e.to_string()))?
                                } else {
                                    Err(DbError::SomethingWentWrong(e.to_string()))?
                                }
                            }
                            _ => Err(DbError::SomethingWentWrong(e.to_string()))?,
                        },
                        _ => Err(DbError::SomethingWentWrong(e.to_string()))?,
                    },
                };
            }
        };
    }

    
    fn verify_password(&self, user: &User, password: &str) -> bool {
        bcrypt::verify(password, &user.password).unwrap_or(false)
    }
}
