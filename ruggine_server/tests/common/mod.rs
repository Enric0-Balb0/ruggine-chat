use std::sync::{atomic::{AtomicU32, Ordering}, Arc};
use tokio::sync::OnceCell;
use ruggine_server::{config::database::{Database, DatabaseTrait}, dto::user_dto::UserLoginDto, entity::user::{NewUser}};

static DATABASE: OnceCell<Arc<Database>> = OnceCell::const_new();

/* pub async fn get_shared_database() -> Arc<Database> {
    init_test_logging();

    DATABASE
        .get_or_init(|| async {
            dotenv::dotenv().ok();

            let database_url = std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "mysql://testuser:testpass@localhost/ruggine_test".to_string());

            let db = Database::init(database_url)
                .await
                .expect("Failed to connect to test database");

            Arc::new(db)
        })
        .await
        .clone()
} */

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


use std::sync::Once;
use ruggine_server::repository::user_repository::UserRepository;

static INIT_LOG: Once = Once::new();

fn init_test_logging() {
    INIT_LOG.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
    });
}