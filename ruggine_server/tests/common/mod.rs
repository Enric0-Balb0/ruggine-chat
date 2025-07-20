// Common file for shared test utilities

use std::sync::Arc;
use std::sync::Once;
use ruggine_server::config::database::{Database, DatabaseTrait};

static INIT: Once = Once::new();

/// Initialize logging for tests (call only once)
pub fn init_test_logging() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

/// Trait for common test helpers
pub trait TestHelper {
    fn create_test_email(prefix: &str) -> String {
        format!("{}@test.com", prefix)
    }
    
    fn create_test_username(prefix: &str) -> String {
        format!("test_{}", prefix)
    }
}

/// Empty implementation of the trait for all types
impl<T> TestHelper for T {}

/// Useful macros for tests
#[macro_export]
macro_rules! assert_user_eq {
    ($expected:expr, $actual:expr) => {
        assert_eq!($expected.email, $actual.email);
        assert_eq!($expected.first_name, $actual.first_name);
        assert_eq!($expected.last_name, $actual.last_name);
        assert_eq!($expected.user_name, $actual.user_name);
        assert_eq!($expected.is_active, $actual.is_active);
    };
}

/// Test constants
pub const TEST_DATABASE_URL: &str = "mysql://root:password@localhost/ruggine_test";
pub const TEST_TIMEOUT_SECONDS: u64 = 30;

/// Utility functions for test setup/teardown
pub async fn setup_test_environment() -> Arc<Database> {
    init_test_logging();
    
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| TEST_DATABASE_URL.to_string());
    
    let database = Database::new_with_url(&database_url).await
        .expect("Failed to connect to test database");
    
    // Run migrations if necessary
    // database.run_migrations().await.expect("Failed to run migrations");
    
    Arc::new(database)
}

pub async fn cleanup_all_test_data(db: &Arc<Database>) {
    // Clean all test data
    let tables = vec!["user", "group_chat", "message", "invitation"];
    
    for table in tables {
        let query = format!("DELETE FROM {} WHERE email LIKE '%@test.com' OR description LIKE '%test%'", table);
        sqlx::query(&query)
            .execute(db.get_pool())
            .await
            .ok(); // Ignore errors if table doesn't exist
    }
}
