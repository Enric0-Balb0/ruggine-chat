use std::sync::{Arc};
use tokio::sync::OnceCell;
use ruggine_server::{config::database::{Database}};
use std::sync::Once;
use axum::Router;
use ruggine_server::config::database::DatabaseTrait;
use ruggine_server::repository::user_repository::UserRepository;
use ruggine_server::routes::{auth, user};
use ruggine_server::state::auth_state::AuthState;
use ruggine_server::state::token_state::TokenState;
use ruggine_server::state::user_state::UserState;

static INIT_LOG: Once = Once::new();
static DATABASE: OnceCell<Arc<Database>> = OnceCell::const_new();
pub async fn get_database() -> Arc<Database> {
    init_test_logging();
    dotenv::dotenv().ok();

    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "mysql://testuser:testpass@localhost/ruggine_test".to_string());

    let db = Database::init(database_url)
        .await
        .expect("Failed to connect to test database");

    Arc::new(db)
}

/// Helper function to cleanup user after test
pub async fn cleanup_user(email: String) {
    let db = get_database().await;
    let repository = UserRepository::new(&db);
    if let Err(e) = repository.delete_by_email(email.clone()).await {
        eprintln!("Cleanup failed for {}: {:?}", email, e);
    }
}

pub async fn create_user_router() -> Router {
    let db = get_database().await;
    let user_state = UserState::new(&db);
    let token_state = TokenState::new(&db);
    user::routes(user_state, token_state)
}

pub async fn create_auth_router() -> Router {
    let db = get_database().await;
    let auth_state = AuthState::new(&db);
    auth::routes().with_state(auth_state)
}

fn init_test_logging() {
    INIT_LOG.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
    });
}