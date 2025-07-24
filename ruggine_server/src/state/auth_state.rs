use crate::config::database::Database;
use crate::repository::user_repository::{self, UserRepositoryTrait};
use crate::service::token_service::{TokenService, TokenServiceTrait};
use crate::service::user_service::{UserService, UserServiceTrait};
use std::sync::Arc;
use crate::config::parameter;

#[derive(Clone)]
pub struct AuthState {
    pub token_service: Arc<dyn TokenServiceTrait + Send + Sync>,
    pub user_repo: Arc<dyn UserRepositoryTrait + Send + Sync>,
    pub user_service: Arc<dyn UserServiceTrait + Send + Sync>,
}

impl AuthState {
    pub fn new(db_conn: &Arc<Database>) -> AuthState {
        Self {
            token_service: Arc::new(TokenService::new(parameter::get("JWT_SECRET"))),
            user_repo: Arc::new(user_repository::UserRepository::new(db_conn)),
            user_service: Arc::new(UserService::new(db_conn)),
        }
    }

    pub fn token_service(&self) -> Arc<dyn TokenServiceTrait + Send + Sync> {
        Arc::clone(&self.token_service)
    }

    pub fn user_repo(&self) -> Arc<dyn UserRepositoryTrait + Send + Sync> {
        Arc::clone(&self.user_repo)
    }

    pub fn user_service(&self) -> Arc<dyn UserServiceTrait + Send + Sync> {
        Arc::clone(&self.user_service)
    }
}
