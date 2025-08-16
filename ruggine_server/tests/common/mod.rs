use std::sync::{Arc};
use tokio::sync::OnceCell;
use ruggine_server::{config::database::{Database}};
use std::sync::Once;
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::json;
use sqlx::PgPool;
use tower::ServiceExt;
use ruggine_server::dto::group_chat_dto::GroupChatReadDto;
use ruggine_server::dto::group_membership_dto::LeaveGroupMembershipDto;
use ruggine_server::entity::group_chat::GroupChat;
use ruggine_server::entity::group_membership;
use ruggine_server::entity::group_membership::{GroupMembership, MemberRole};
use ruggine_server::entity::user::User;
use ruggine_server::entity::invitation::{Invitation, NewInvitation};
use ruggine_server::entity::text_message::{TextMessage, NewTextMessage};
use ruggine_server::factory::group_chat_factory::GroupChatFactory;
use ruggine_server::factory::user_factory::UserFactory;
use ruggine_server::factory::text_message_factory::TextMessageFactory;
use ruggine_server::model::group_membership_model::GroupMembershipWithInvitationRow;
use ruggine_server::repository::group_chat_repository::{GroupChatRepository, GroupChatRepositoryTrait};
use ruggine_server::repository::group_membership_repository::{GroupMembershipRepository, GroupMembershipRepositoryTrait};
use ruggine_server::repository::user_repository::{UserRepository, UserRepositoryTrait};
use ruggine_server::repository::invitation_repository::{InvitationRepository, InvitationRepositoryTrait};
use ruggine_server::repository::text_message_repository::{TextMessageRepository, TextMessageRepositoryTrait};
use ruggine_server::routes::{auth_route, user_route, group_chat_route, invitation_route, group_membership_route, text_message_route};
use ruggine_server::service::user_service::{UserService, UserServiceTrait};
use ruggine_server::state::auth_state::AuthState;
use ruggine_server::state::group_chat_state::GroupChatState;
use ruggine_server::state::group_membership_state::GroupMembershipState;
use ruggine_server::state::invitation_state::InvitationState;
use ruggine_server::state::token_state::TokenState;
use ruggine_server::state::user_state::UserState;
use ruggine_server::state::text_message_state::TextMessageState;
use ruggine_server::utils::service_initializer::ServiceInitializer;

static INIT_LOG: Once = Once::new();
static DB_POOL: OnceCell<PgPool> = OnceCell::const_new();


pub async fn get_database() -> Arc<Database> {
    init_test_logging();
    dotenv::dotenv().ok();

    // Inizializza il pool una volta sola in modo async-safe
    let pool = DB_POOL
        .get_or_init(|| async {
            let database_url = std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://testuser:testpass@localhost/ruggine_test".to_string());

            PgPool::connect(&database_url)
                .await
                .expect("Failed to connect to test database")
        })
        .await;

    Arc::new(Database { pool: pool.clone() })
}

/// Helper function to clea nup user after test
pub async fn cleanup_user_by_email(email: String) {
    let db = get_database().await;
    let repository = UserRepository::new(&db);
    if let Err(e) = repository.delete_by_email(email.clone()).await {
        panic!("Cleanup failed for {}: {:?}", email, e);
    }
}

pub async fn cleanup_user(id: i32) {
    let db = get_database().await;
    let repository = UserRepository::new(&db);
    if let Err(e) = repository.delete_by_id(id).await {
        panic!("Cleanup failed for {}: {:?}", id, e);
    }
}

pub async fn create_user_router() -> Router {
    let db = get_database().await;
    let user_state = UserState::new(&db);
    let token_state = TokenState::new(&db);
    user_route::routes(user_state, token_state)
}

pub async fn create_auth_router() -> Router {
    let db = get_database().await;
    let auth_state = AuthState::new(&db);
    auth_route::routes().with_state(auth_state)
}

pub async fn create_group_chat_router() -> Router {
    let db = get_database().await;
    let group_chat_state = GroupChatState::new(&db);
    let token_state = TokenState::new(&db);
    group_chat_route::routes(group_chat_state, token_state)
}

pub async fn create_invitation_router() -> Router {
    let db = get_database().await;
    let invitation_state = InvitationState::new(&db);
    let token_state = TokenState::new(&db);
    invitation_route::routes(invitation_state, token_state)
}

pub async fn create_group_membership_router() -> Router {
    let db = get_database().await;
    let group_membership_state = GroupMembershipState::new(&db);
    let token_state = TokenState::new(&db);
    group_membership_route::routes(group_membership_state, token_state)
}

pub async fn create_text_message_router() -> Router {
    let db = get_database().await;
    let text_message_state = TextMessageState::new(&db);
    let token_state = TokenState::new(&db);
    text_message_route::routes(text_message_state, token_state)
}

/// Helper function to create the full application router for e2e tests
pub async fn create_full_router() -> Router {
    let db = get_database().await;
    ruggine_server::routes::root::routes(db)
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

/// Helper function to create a test user with default password
pub async fn create_test_user(prefix: &str) -> (User, String) {
    let password = "testpassword123".to_string();
    let user = create_test_user_with_password(prefix, password.clone()).await;
    (user, password)
}

// Helper function to log in and get token
pub async fn login_and_get_token(email: String, password: String) -> String {
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

/// Public helper function to log in and get token for tests
pub async fn login_and_get_token_for_user(user: &User, password: &str) -> String {
    login_and_get_token(user.email.clone(), password.to_string()).await
}

pub async fn create_login_and_get_token(prefix: String) -> (User, String, String) {
    let (user, password) = create_test_user(prefix.as_str()).await;
    let token = login_and_get_token_for_user(&user, &password).await;
    (user, password, token)
}

/// Helper function to create a real group chat in the database
pub async fn create_test_group_chat(prefix: &str, created_by: i32) -> GroupChat {
    let db = get_database().await;
    let repository = GroupChatRepository::new(&db);

    let new_group = GroupChatFactory::unique_fake_new_group_chat(prefix, created_by);

    let inserted_id = repository.insert(new_group.clone()).await
        .expect("Failed to insert test group chat");

    // Get the created group from database
    let group_option = repository.find_by_id(inserted_id).await;
    assert!(group_option.is_ok(), "Group chat not found in database");
    group_option.unwrap()
}

pub async fn create_test_group_chat_with_invitation_and_membership(prefix: &str, created_by: i32) -> GroupChatReadDto {
    let db = get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let group_chat_service = service_init.group_chat_service();

    let fake_group_chat_create_dto = GroupChatFactory::unique_fake_group_chat_create_dto(prefix);

    let group_chat = group_chat_service.create(fake_group_chat_create_dto, created_by).await.unwrap();

    group_chat
}

pub async fn add_test_user_to_a_group(user_id: i32, group_chat: &GroupChatReadDto) -> GroupMembershipWithInvitationRow {
    let invitation = create_test_invitation(group_chat.created_by, user_id, group_chat.id).await;
    let membership = create_test_group_membership(invitation.id, user_id).await;
    return membership;
}

pub async fn cleanup_group_chat(group_id: i32) {
    let db = get_database().await;
    let group_repo = GroupChatRepository::new(&db);
    if let Err(e) = group_repo.delete_by_id(group_id).await {
        eprintln!("Failed to cleanup group {}: {:?}", group_id, e);
    }
    
}

pub async fn leave_user_from_a_group(user_id: i32, group_id: i32) {
    let db = get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let group_membership_service = service_init.group_membership_service();
    let membership = group_membership_service.find_by_user_id_and_group_id(user_id, group_id).await.unwrap();
    group_membership_service.leave_group(LeaveGroupMembershipDto {id: membership.id}, user_id).await.unwrap();
}

/// Helper function to create a real group chat state with database connections
pub async fn create_group_chat_state() -> GroupChatState {
    let db = get_database().await;
    let service_init = ServiceInitializer::new(&db);
    GroupChatState {
        group_chat_service: service_init.group_chat_service(),
        user_service: service_init.user_service(),
    }
}

/// Helper function to create a real invitation state with database connections
pub async fn create_invitation_state() -> InvitationState {
    let db = get_database().await;
    InvitationState::new(&db)
}

/// Helper function to create a real text message state with database connections
pub async fn create_text_message_state() -> TextMessageState {
    let db = get_database().await;
    TextMessageState::new(&db)
}


/// Helper function to create a test invitation in the database
pub async fn create_test_invitation(from_user_id: i32, to_user_id: i32, group_chat_id: i32) -> Invitation {
    let db = get_database().await;
    let repository = InvitationRepository::new(&db);

    let new_invitation = NewInvitation {
        from_user_id,
        to_user_id,
        group_chat_id,
        role_at_join: MemberRole::Member,
    };

    let inserted_id = repository.insert(new_invitation.clone()).await
        .expect("Failed to insert test invitation");

    // Get the created invitation from database
    let invitation_result = repository.find_by_id_and_user_id(inserted_id, to_user_id).await;
    assert!(invitation_result.is_ok(), "Invitation not found in database");
    invitation_result.unwrap()
}

/// Helper function to create a test admin invitation in the database
pub async fn create_test_admin_invitation(from_user_id: i32, to_user_id: i32, group_chat_id: i32) -> Invitation {
    let db = get_database().await;
    let repository = InvitationRepository::new(&db);

    let new_invitation = NewInvitation {
        from_user_id,
        to_user_id,
        group_chat_id,
        role_at_join: MemberRole::Admin,
    };

    let inserted_id = repository.insert(new_invitation.clone()).await
        .expect("Failed to insert test invitation");

    // Get the created invitation from database
    let invitation_result = repository.find_by_id_and_user_id(inserted_id, to_user_id).await;
    assert!(invitation_result.is_ok(), "Invitation not found in database");
    invitation_result.unwrap()
}

/// Helper function to clean up invitation after test
pub async fn cleanup_invitation(invitation_id: i32) {
    let db = get_database().await;
    let repository = InvitationRepository::new(&db);

    if let Err(e) = repository.delete_by_id(invitation_id).await {
        panic!("Cleanup failed for invitation with id {}: {:?}", invitation_id, e);
    }
    
}

pub async fn create_test_group_membership(
    invitation_id: i32,
    user_id: i32,
) -> GroupMembershipWithInvitationRow {
    let db = get_database().await;
    let repository = GroupMembershipRepository::new(&db);

    let new_membership = ruggine_server::factory::group_membership_factory::GroupMembershipFactory::fake_new_group_membership_with_id(invitation_id);
    let inserted_id = repository.insert(new_membership.clone()).await
        .expect("Failed to insert test group membership");

    // Get the created membership from database
    let membership_result = repository.find_by_id_and_user_id(inserted_id, user_id).await;
    assert!(membership_result.is_ok(), "Group membership not found in database");
    membership_result.unwrap()
}

pub async fn create_test_admin_group_membership(
    invitation_id: i32,
    user_id: i32,
) -> GroupMembershipWithInvitationRow {
    let db = get_database().await;
    let repository = GroupMembershipRepository::new(&db);

    let new_membership = ruggine_server::factory::group_membership_factory::GroupMembershipFactory::fake_new_admin_group_membership_with_id(invitation_id);
    let inserted_id = repository.insert(new_membership.clone()).await
        .expect("Failed to insert test group membership");

    // Get the created membership from database
    let membership_result = repository.find_by_id_and_user_id(inserted_id, user_id).await;
    assert!(membership_result.is_ok(), "Group membership not found in database");
    membership_result.unwrap()
}

pub async fn cleanup_group_membership(membership_id: i32) {
    let db = get_database().await;
    let repository = GroupMembershipRepository::new(&db);

    if let Err(e) = repository.delete_by_id(membership_id).await {
        panic!("Cleanup failed for group membership with id {}: {:?}", membership_id, e);
    }
}

pub async fn cleanup_test_user_from_a_group_chat(user_id: i32, group_chat_id: i32) {
    let db = get_database().await;
    let service_init = ServiceInitializer::new(&db);
    let group_membership_service = service_init.group_membership_service();
    let group_membership = group_membership_service.find_by_user_id_and_group_id(user_id, group_chat_id).await.unwrap();
    cleanup_group_membership(group_membership.id).await;
    cleanup_invitation(group_membership.invitation_id).await;
}

pub async fn cleanup_group_membership_by_invitation_id(invitation_id: i32) {
    let db = get_database().await;
    let repository = GroupMembershipRepository::new(&db);

    if let Err(e) = repository.delete_by_invitation_id(invitation_id).await {
        panic!("Cleanup failed for group membership with invitation_id {}: {:?}", invitation_id, e);
    }
}

/// Helper function to create a test text message in the database
pub async fn create_test_text_message(sender_id: i32, group_chat_id: i32, content: Option<String>) -> TextMessage {
    let db = get_database().await;
    let repository = TextMessageRepository::new(&db);

    let new_message = NewTextMessage {
        content: content.unwrap_or_else(|| "Test message content".to_string()),
        sender_id,
        group_chat_id,
    };

    let inserted_id = repository.insert(new_message).await.unwrap();

    // Get the created message from database
    let message = repository.find(inserted_id).await;
    assert!(message.is_ok(), "Text message not found in database");
    message.unwrap()
}

/// Helper function to create multiple test text messages for a group
pub async fn create_test_text_messages_for_group(group_chat_id: i32, sender_id: i32, count: usize) -> Vec<TextMessage> {
    let mut messages = Vec::new();
    for i in 0..count {
        let content = format!("Test message {} for group {}", i + 1, group_chat_id);
        let message = create_test_text_message(sender_id, group_chat_id, Some(content)).await;
        messages.push(message);
        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    messages
}

/// Helper function to create multiple test text messages with different senders
pub async fn create_test_text_messages_multi_sender(group_chat_id: i32, sender_ids: Vec<i32>) -> Vec<TextMessage> {
    let mut messages = Vec::new();
    for (i, sender_id) in sender_ids.iter().enumerate() {
        let content = format!("Message from user {} in group {}", sender_id, group_chat_id);
        let message = create_test_text_message(*sender_id, group_chat_id, Some(content)).await;
        messages.push(message);
        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    messages
}

/// Helper function to create multiple test users
pub async fn create_test_users(prefix: &str, count: usize) -> Vec<(User, String)> {
    let mut users = Vec::new();
    for i in 0..count {
        let username = format!("{}_user_{}", prefix, i + 1);
        let user_data = create_test_user(&username).await;
        users.push(user_data);
    }
    users
}

pub async fn cleanup_test_users(user_ids: Vec<i32>) {
    for user_id in user_ids {
        cleanup_user(user_id).await;
    }
}

pub async fn create_test_users_for_a_group(
    prefix: &str,
    count: usize,
    group_chat: &GroupChatReadDto,
) -> Vec<(User, String, GroupMembershipWithInvitationRow)> {
    let users = create_test_users(prefix, count).await;
    let mut response = Vec::new();

    for user in users {
        let membership = add_test_user_to_a_group(user.0.id, group_chat).await;
        response.push((user.0, user.1, membership));
    }

    response
}

pub async fn cleanup_test_users_from_a_group_chat(user_ids: Vec<i32>, group_chat_id: i32) {
    for user_id in user_ids {
        cleanup_test_user_from_a_group_chat(user_id, group_chat_id).await;
    }
}

/// Helper function to cleanup a text message from database
pub async fn cleanup_text_message(message_id: i32) {
    let db = get_database().await;
    let repository = TextMessageRepository::new(&db);
    if let Err(e) = repository.delete_by_id(message_id).await {
        panic!("Cleanup failed for {}: {:?}", message_id, e);
    }
}

/// Helper function to cleanup multiple text messages from database
pub async fn cleanup_text_messages(message_ids: Vec<i32>) {
    for message_id in message_ids {
        cleanup_text_message(message_id).await;
    }
}

fn init_test_logging() {
    INIT_LOG.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
    });
}