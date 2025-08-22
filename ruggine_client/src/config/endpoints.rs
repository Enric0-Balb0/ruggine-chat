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
    pub const INVITATION_UPDATE: &'static str = "/invitation/update_status";
    pub const INVITATION_BY_USER: &'static str = "/invitation/user";
    pub const INVITATION_BY_ID: &'static str = "/invitation"; // + /{id}
    
    // Message endpoints (future)
    pub const MESSAGE_SEND: &'static str = "/messages";
    pub const MESSAGE_HISTORY: &'static str = "/messages/history";
    pub const MESSAGE_UNREAD_COUNTS: &'static str = "/messages/unread_counts";
    
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
}

/// WebSocket endpoints for real-time features
pub struct WebSocketEndpoints;

impl WebSocketEndpoints {
    pub const CHAT_WEBSOCKET: &'static str = "/ws/chat";
    pub const NOTIFICATIONS_WEBSOCKET: &'static str = "/ws/notifications";
    
    pub fn chat_websocket_url(base_url: &str, group_id: &str) -> String {
        format!("{}{}/group/{}", base_url.replace("http", "ws"), Self::CHAT_WEBSOCKET, group_id)
    }
}
