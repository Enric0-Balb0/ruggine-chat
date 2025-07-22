use mockall::automock;
use crate::config::database::Database;
use crate::repository::user_repository::{UserRepository, UserRepositoryTrait};
use crate::service::token_service::{TokenService, TokenServiceTrait};
use std::sync::Arc;

#[derive(Clone)]
pub struct TokenState {
    pub token_service: Arc<dyn TokenServiceTrait + Send + Sync>,
    pub user_repo: Arc<dyn UserRepositoryTrait + Send + Sync>,
}

#[automock]
impl TokenState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            token_service: Arc::new(TokenService::new()),
            user_repo: Arc::new(UserRepository::new(db_conn)),
        }
    }
}

