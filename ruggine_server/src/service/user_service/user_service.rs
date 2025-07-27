use std::sync::Arc;
use crate::config::database::Database;
use crate::repository::user_repository::{UserRepository, UserRepositoryTrait};

#[derive(Clone)]
pub struct UserService {
    pub(crate) user_repo: Arc<dyn UserRepositoryTrait>,
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