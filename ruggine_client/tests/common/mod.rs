// Common testing utilities for ruggine_client
// CLIENT-FOCUSED TESTING: UI state, validation, data transformation

pub mod factories;

use factories::*;
use std::sync::Once;

static INIT_LOG: Once = Once::new();

/// Factory trait for testing utilities
pub trait TestFactoryTrait {
    fn new() -> Self;
}

/// Main TestFactory for delegation to specialized factories
pub struct TestFactory;

impl TestFactory {
    // User factory delegation methods
    pub fn mock_user_profile() -> ruggine_client_ui::types::user::UserProfile {
        UserFactory::mock_user_profile()
    }
    
    pub fn unique_user_register_request(prefix: &str) -> ruggine_client_ui::types::user::UserRegisterRequest {
        UserFactory::unique_user_register_request(prefix)
    }
    
    pub fn get_unique_user_info(prefix: &str) -> (String, String, String) {
        BaseFactory::get_unique_user_info(prefix)
    }
    
    // Auth factory delegation methods
    pub fn mock_token_response() -> ruggine_client_ui::types::auth::TokenResponse {
        AuthFactory::mock_token_response()
    }
    
    pub fn mock_token_response_with_exp(exp: i64) -> ruggine_client_ui::types::auth::TokenResponse {
        AuthFactory::mock_token_response_with_exp(exp)
    }
    
    pub fn unique_login_request(prefix: &str) -> ruggine_client_ui::types::auth::LoginRequest {
        AuthFactory::unique_login_request(prefix)
    }
    
    // Group factory delegation methods
    pub fn valid_group_create_request(prefix: &str) -> ruggine_client_ui::types::group::GroupChatCreateRequest {
        GroupFactory::minimal_group_create_request(prefix)
    }
    
    pub fn minimal_group_create_request(prefix: &str) -> ruggine_client_ui::types::group::GroupChatCreateRequest {
        GroupFactory::minimal_group_create_request(prefix)
    }
    
    pub fn custom_group_create_request(name: &str, description: &str) -> ruggine_client_ui::types::group::GroupChatCreateRequest {
        GroupFactory::custom_group_create_request(name, description)
    }
    
    pub fn mock_group_chat() -> ruggine_client_ui::types::group::GroupChat {
        GroupFactory::mock_group_chat()
    }
    
    pub fn mock_group_membership(prefix: &str) -> ruggine_client_ui::types::membership::GroupMembership {
        GroupFactory::mock_group_membership(prefix)
    }
    
    pub fn mock_multiple_group_memberships(count: usize, prefix: &str) -> Vec<ruggine_client_ui::types::membership::GroupMembership> {
        (0..count).map(|i| {
            let mut membership = GroupFactory::mock_group_membership(&format!("{}_{}", prefix, i));
            membership.id = i as i32 + 1;
            membership.group_chat_id = i as i32 + 1;
            membership
        }).collect()
    }
    
    // Message factory delegation methods
    pub fn mock_message(prefix: &str) -> ruggine_client_ui::types::message::Message {
        message_factory::MessageFactory::mock_message(prefix)
    }
    
    pub fn mock_message_with_params(message_id: i32, content: &str, group_id: i32, sender_id: i32) -> ruggine_client_ui::types::message::Message {
        message_factory::MessageFactory::mock_message_with_params(message_id, content, group_id, sender_id)
    }
    
    pub fn mock_message_create_request(content: &str, group_id: i32) -> ruggine_client_ui::types::message::TextMessageCreateRequest {
        message_factory::MessageFactory::mock_message_create_request(content, group_id)
    }
    
    pub fn mock_message_read_dto(prefix: &str) -> ruggine_client_ui::types::message::TextMessageReadDto {
        // Since MessageFactory returns Message, we need to convert to TextMessageReadDto
        let msg = message_factory::MessageFactory::mock_message(prefix);
        use ruggine_client_ui::types::message::TextMessageReadDto;
        TextMessageReadDto {
            id: msg.id,
            content: msg.content,
            sender_id: msg.sender_id,
            group_chat_id: msg.group_chat_id,
            sent_at: msg.sent_at.to_rfc3339(),
        }
    }
    
    pub fn mock_multiple_messages(count: usize, prefix: &str, group_id: i32) -> Vec<ruggine_client_ui::types::message::Message> {
        message_factory::MessageFactory::mock_multiple_messages(count, prefix, group_id)
    }
    
    pub fn mock_message_info_read_dto(message_id: i32, read_at_id: i32) -> ruggine_client_ui::types::message::TextMessageInfoReadDto {
        use ruggine_client_ui::types::message::TextMessageInfoReadDto;
        TextMessageInfoReadDto {
            id: read_at_id,
            user_id: read_at_id,
            text_message_id: message_id,
            sent_at: Some(chrono::Utc::now().to_rfc3339()),
            read_at: None, // Default to unread
        }
    }
    
    pub fn mock_read_update_request(message_id: i32) -> ruggine_client_ui::types::message::TextMessageInfoReadAtDtoUpdate {
        message_factory::MessageFactory::mock_read_update_request(message_id)
    }
    
    pub fn mock_multiple_read_updates(message_ids: &[i32]) -> Vec<ruggine_client_ui::types::message::TextMessageInfoReadAtDtoUpdate> {
        message_factory::MessageFactory::mock_multiple_read_updates(message_ids.to_vec())
    }
    
    pub fn mock_paginated_message_response(prefix: &str, count: usize, has_more: bool) -> ruggine_client_ui::types::message::PaginatedTextMessageResponse {
        // Convert from MessagePage to PaginatedTextMessageResponse
        let page = message_factory::MessageFactory::mock_paginated_message_response(prefix, count, has_more);
        use ruggine_client_ui::types::message::{PaginatedTextMessageResponse, PaginationMetadataDto, TextMessageReadDto};
        
        let messages = page.data.into_iter()
            .map(|msg| TextMessageReadDto {
                id: msg.id,
                content: msg.content,
                sender_id: msg.sender_id,
                group_chat_id: msg.group_chat_id,
                sent_at: msg.sent_at.to_rfc3339(),
            })
            .collect();
            
        PaginatedTextMessageResponse {
            data: messages,
            pagination: PaginationMetadataDto {
                has_more: page.pagination.has_more,
                next_cursor: page.pagination.next_cursor.map(|dt| dt.to_rfc3339()),
                page_size: page.pagination.page_size,
                total_count: page.pagination.total_count,
            },
        }
    }
    
    pub fn mock_message_page(prefix: &str, count: usize, has_more: bool) -> ruggine_client_ui::types::message::MessagePage {
        message_factory::MessageFactory::mock_paginated_message_response(prefix, count, has_more)
    }
    
    // WebSocket methods delegation
    pub fn mock_ws_group_left_event(user_id: i32) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_group_left_event(user_id)
    }
    
    pub fn mock_ws_new_message_event(message_id: i32, group_id: i32, sender_id: i32, sender_username: &str, content: &str) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_new_message_event(message_id, group_id, sender_id, sender_username, content)
    }
    
    pub fn mock_ws_group_joined_event(user_id: i32) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_group_joined_event(user_id)
    }
    
    pub fn mock_ws_info_notification(message: &str) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_info_notification(message)
    }
    
    pub fn mock_ws_success_notification(message: &str) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_success_notification(message)
    }
    
    pub fn mock_ws_warning_notification(message: &str) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_warning_notification(message)
    }
    
    pub fn mock_ws_error_notification(message: &str) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_error_notification(message)
    }
    
    pub fn mock_ws_connect_control() -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_connect_control()
    }
    
    pub fn mock_ws_disconnect_control() -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_disconnect_control()
    }
    
    pub fn mock_ws_ping_control() -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_ping_control()
    }
    
    pub fn mock_ws_pong_control() -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_pong_control()
    }
    
    pub fn mock_ws_ack_control(request_id: Option<&str>) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_ack_control(request_id)
    }
    
    pub fn mock_ws_error_control(code: u16, message: &str, request_id: Option<&str>) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_error_control(code, message, request_id)
    }
    
    pub fn mock_ws_validation_error_control(field: &str, message: &str, request_id: Option<&str>) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        // Create a validation error with code 400 for generic validation errors
        WebSocketFactory::mock_ws_validation_error_control(400, &format!("{}: {}", field, message), request_id)
    }
    
    pub fn mock_ws_error(code: u16, message: &str, request_id: Option<&str>) -> ruggine_client_ui::types::message_ws::WsError {
        WebSocketFactory::mock_ws_error(code, message, request_id)
    }
    
    pub fn mock_ws_status_progression() -> Vec<ruggine_client_ui::types::message_ws::WsStatus> {
        WebSocketFactory::mock_ws_status_progression()
    }
    
    pub fn mock_ws_error_status_progression() -> Vec<ruggine_client_ui::types::message_ws::WsStatus> {
        WebSocketFactory::mock_ws_error_status_progression()
    }
    
    pub fn mock_ws_message_flow(group_id: i32, user_id: i32) -> Vec<ruggine_client_ui::types::message_ws::WebSocketMessage> {
        WebSocketFactory::mock_ws_message_flow(group_id, user_id)
    }
    
    pub fn mock_ws_join_request() -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_join_request()
    }
    
    pub fn mock_ws_success_response(request_id: &str) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_success_response(request_id)
    }
    
    pub fn mock_ws_error_response(request_id: &str, message: &str) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_error_response(request_id, message)
    }
    
    // Additional WebSocket methods
    pub fn mock_ws_leave_request() -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_leave_request()
    }
    
    pub fn mock_ws_new_message_request(group_id: i32, content: &str) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_new_message_request(group_id, content)
    }
    
    pub fn mock_ws_test_request(message: &str) -> ruggine_client_ui::types::message_ws::WebSocketMessage {
        WebSocketFactory::mock_ws_test_request(message)
    }
    
    // Mock grouped membership details for hooks test
    pub fn mock_group_membership_with_details() -> ruggine_client_ui::hooks::GroupMembershipWithDetails {
        use ruggine_client_ui::hooks::GroupMembershipWithDetails;
        use ruggine_client_ui::types::membership::GroupMembership;
        use ruggine_client_ui::types::group::GroupChat;
        use ruggine_client_ui::types::invitation::{MemberRole, MembershipStatus};
        use ruggine_client_ui::types::membership::CurrentAction;
        use chrono::{DateTime, Utc};
        
        let unique_id = BaseFactory::get_unique_id() as i32;
        
        let membership = GroupMembership {
            id: unique_id,
            user_id: unique_id,
            group_chat_id: unique_id,
            role: MemberRole::Member,
            joined_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: unique_id,
            current_action: CurrentAction::Waiting,
            group_name: Some(format!("group{}", unique_id)),
            user_name: Some(format!("user{}", unique_id)),
        };
        
        let group_chat = GroupChat {
            id: unique_id,
            name: format!("group{}", unique_id),
            description: format!("A test group {}", unique_id),
            created_by: unique_id,
            created_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            updated_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            member_count: Some(1),
            is_active: true,
        };
        
        GroupMembershipWithDetails {
            membership,
            group_details: Some(group_chat),
        }
    }
    
    // Base utility methods
    pub fn generate_uuid() -> String {
        BaseFactory::generate_uuid()
    }
    
    // Additional user methods
    pub fn user_with_details(id: i32, username: &str, email: &str) -> ruggine_client_ui::types::user::UserProfile {
        UserFactory::user_with_details(id, username, email)
    }
    
    pub fn basic_user_register_request() -> ruggine_client_ui::types::user::UserRegisterRequest {
        UserFactory::basic_user_register_request()
    }
}

/// Spawn runtime for async tests in Leptos context
pub async fn spawn_test_runtime<F, R>(test_fn: F) -> R
where
    F: std::future::Future<Output = R>,
{
    test_fn.await
}

/// Initialize test logging for client tests
pub fn init_test_logging() {
    INIT_LOG.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
    });
}

// CLIENT-SIDE constants for testing
pub const MOCK_SERVER_URL: &str = "http://localhost:3000";
pub const MOCK_JWT_TOKEN: &str = "mock_jwt_token_12345";

pub fn mock_user_profile_json() -> &'static str {
    r#"{
        "data": {
            "id": 1,
            "email": "test@example.com",
            "first_name": "Test",
            "last_name": "User",
            "username": "testuser",
            "user_status": "Active",
            "user_type": "User",
            "current_action": "Online",
            "birthday": "1990-01-01",
            "address": "123 Test St",
            "gender": "male",
            "created_at": "2025-01-01T00:00:00Z",
            "updated_at": "2025-01-01T00:00:00Z",
            "last_login": "2025-01-01T00:00:00Z"
        },
        "success": true
    }"#
}

pub fn mock_token_response_json() -> &'static str {
    r#"{
        "data": {
            "token": "mock_jwt_token_12345",
            "iat": 1640995200,
            "exp": 1641081600
        },
        "success": true
    }"#
}
