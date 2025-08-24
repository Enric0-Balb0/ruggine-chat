/// API endpoints configuration
pub struct ApiEndpoints;

impl ApiEndpoints {
    // Authentication endpoints
    pub const AUTH_LOGIN: &'static str = "/auth/login";
    pub const AUTH_LOGOUT: &'static str = "/auth/logout";
    pub const AUTH_REFRESH: &'static str = "/auth/refresh";
    pub const AUTH_VERIFY: &'static str = "/auth/verify";

    // User endpoints
    pub const USER_REGISTER: &'static str = "/user/register";
    pub const USER_PROFILE: &'static str = "/user/profile";
    pub const USER_UPDATE_PROFILE: &'static str = "/user/profile";
    pub const USER_CHANGE_PASSWORD: &'static str = "/user/password";
    pub const USER_GROUPS: &'static str = "/user/groups";
    pub const USER_BY_ID: &'static str = "/user"; // + /{id}
    pub const USER_BY_USERNAME: &'static str = "/user/username"; // + /{username}
    
    // Group chat endpoints
    pub const GROUP_CREATE: &'static str = "/group_chat/create";
    pub const GROUP_BY_ID: &'static str = "/group_chat"; // + /{id}
    pub const GROUP_UPDATE: &'static str = "/group_chat"; // + /{id}
    pub const GROUP_DELETE: &'static str = "/group_chat"; // + /{id}
    pub const GROUP_PARTICIPANTS: &'static str = "/group_chat"; // + /{id}/participants
    pub const GROUP_MESSAGES: &'static str = "/group_chat"; // + /{id}/messages
    
    // Group membership endpoints
    pub const GROUP_MEMBERSHIP_BY_USER: &'static str = "/group_membership/user";
    pub const GROUP_MEMBERSHIP_BY_ID: &'static str = "/group_membership"; // + /{id}
    pub const GROUP_MEMBERSHIP_BY_GROUP_CHAT: &'static str = "/group_membership/group_chat"; // + /{group_id}
    pub const GROUP_MEMBERSHIP_LEAVE: &'static str = "/group_membership/leave";

    // Invitation endpoints
    pub const INVITATION_SEND: &'static str = "/invitation/send";
    pub const INVITATION_UPDATE: &'static str = "/invitation/update-status";
    pub const INVITATION_BY_USER: &'static str = "/invitation/user";
    pub const INVITATION_BY_ID: &'static str = "/invitation"; // + /{id}
    
    // Message endpoints
    pub const TEXT_MESSAGE_CREATE: &'static str = "/messages";
    
    // Utility methods for dynamic endpoints
    pub fn group_by_id(group_id: &str) -> String {
        format!("{}/{}", Self::GROUP_BY_ID, group_id)
    }

    pub fn group_membership_by_group_chat(group_id: &str) -> String {
        format!("{}/{}", Self::GROUP_MEMBERSHIP_BY_GROUP_CHAT, group_id)
    }
    
    pub fn group_participants(group_id: &str) -> String {
        format!("{}/{}/participants", Self::GROUP_BY_ID, group_id)
    }
    
    pub fn group_messages(group_id: &str) -> String {
        format!("{}/{}/messages", Self::GROUP_BY_ID, group_id)
    }

    pub fn invitation_by_id(invitation_id: &str) -> String {
        format!("{}/{}", Self::INVITATION_BY_ID, invitation_id)
    }
    

    pub fn user_by_id(user_id: &str) -> String {
        format!("{}/{}", Self::USER_BY_ID, user_id)
    }

    pub fn user_by_username(username: &str) -> String {
        format!("{}/{}", Self::USER_BY_USERNAME, username)
    }

    pub fn text_messages_by_group(group_id: &str) -> String {
        format!("/messages/group/{}", group_id)
    }
}

/// WebSocket endpoints for real-time features
pub struct WebSocketEndpoints;

impl WebSocketEndpoints {
    pub const CHAT_WEBSOCKET: &'static str = "/ws/chat";
    pub const NOTIFICATIONS_WEBSOCKET: &'static str = "/ws/notifications";

    /// The correct group WebSocket endpoint as used by the server
    pub const GROUP_WEBSOCKET: &'static str = "/api/ws/group";

    /// Returns the full WebSocket URL for group messaging, including the token as a query parameter
    pub fn group_websocket_url(base_url: &str, token: &str) -> String {
        // base_url should be like http://127.0.0.1:8002 or https://...
        let ws_base = if base_url.starts_with("https") {
            base_url.replacen("https", "wss", 1)
        } else {
            base_url.replacen("http", "ws", 1)
        };
        format!("{}{}?token={}", ws_base, Self::GROUP_WEBSOCKET, token)
    }

    pub fn chat_websocket_url(base_url: &str, group_id: &str) -> String {
        format!("{}{}/group/{}", base_url.replace("http", "ws"), Self::CHAT_WEBSOCKET, group_id)
    }
}
