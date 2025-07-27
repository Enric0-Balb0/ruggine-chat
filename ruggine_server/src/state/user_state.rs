use mockall::automock;

use crate::config::database::Database;
use crate::repository::user_repository::{UserRepository, UserRepositoryTrait};
use crate::service::user_service::{UserService};
use crate::service::user_service::UserServiceTrait;

use std::sync::Arc;

#[derive(Clone)]
pub struct UserState {
    pub user_service: Arc<dyn UserServiceTrait>,
    pub user_repo: Arc<dyn UserRepositoryTrait>,
}

#[automock]
impl UserState {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        let user_repo = Arc::new(UserRepository::new(db_conn));
        let user_service = Arc::new(UserService::new(db_conn));

        Self {
            user_service,
            user_repo,
        }
    }
}
