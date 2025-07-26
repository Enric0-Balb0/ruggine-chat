use std::sync::{Arc};
use tokio::sync::OnceCell;
use ruggine_server::{config::database::{Database}};
use std::sync::Once;
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::json;
use tower::ServiceExt;
use ruggine_server::config::database::DatabaseTrait;
use ruggine_server::entity::user::User;
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::routes::{auth, user};
use ruggine_server::service::user_service::{UserService, UserServiceTrait};
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

/// Helper function to create a real user in the database
async fn create_test_user_with_password(prefix: &str, password: String) -> User {
    let db = get_database().await;
    let user_service = UserService::new(&db);
    let repository = UserRepository::new(&db);

    let mut user_dto = UserFactory::unique_fake_user_register_dto(prefix);
    user_dto.password = password;
    let create_result = user_service.create_user(user_dto.clone()).await;
    assert!(create_result.is_ok(), "Failed to create user for profile test");

    // Get the created user from database
    let user_option = repository.find_by_email(user_dto.email.clone()).await;
    assert!(user_option.is_some(), "User not found in database");
    user_option.unwrap()
}

// Helper function to log in and get token
async fn login_and_get_token(email: String, password: String) -> String {
    let auth_app = create_auth_router().await;

    let login_payload = json!({
            "email": email,
            "password": password
        });

    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/json")
        .body(Body::from(login_payload.to_string()))
        .unwrap();

    let response = auth_app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK, "Login should succeed");

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_text = String::from_utf8(body.to_vec()).unwrap();
    let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

    // Verify response structure contains user data
    assert!(response_json.get("data").is_some(), "Response should contain data field");
    let data = &response_json["data"];

    data["token"].as_str().unwrap().to_string()
}


fn init_test_logging() {
    INIT_LOG.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
    });
}