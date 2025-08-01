/// Application-wide constants
pub struct AppConstants;

impl AppConstants {
    /// Default server URL for API calls
    pub const DEFAULT_SERVER_URL: &'static str = "http://localhost:8002";
    
    /// Default timeout for HTTP requests (in seconds)
    pub const HTTP_TIMEOUT_SECONDS: u64 = 30;
    
    /// Maximum retry attempts for failed requests
    pub const MAX_RETRY_ATTEMPTS: u32 = 3;
    
    /// Delay between retry attempts (in milliseconds)
    pub const RETRY_DELAY_MS: u64 = 1000;
}

/// Authentication related constants
pub struct AuthConstants;

impl AuthConstants {
    /// Minimum password length for registration
    pub const MIN_PASSWORD_LENGTH: usize = 8;
    
    /// Maximum password length
    pub const MAX_PASSWORD_LENGTH: usize = 128;
    
    /// Token refresh threshold (minutes before expiry)
    pub const TOKEN_REFRESH_THRESHOLD_MINUTES: i64 = 5;
    
    /// Maximum login attempts before lockout
    pub const MAX_LOGIN_ATTEMPTS: u32 = 5;
}

/// UI/UX related constants
pub struct UiConstants;

impl UiConstants {
    /// Default items per page for pagination
    pub const DEFAULT_PAGE_SIZE: u32 = 20;
    
    /// Maximum items per page
    pub const MAX_PAGE_SIZE: u32 = 100;
    
    /// Debounce delay for search input (milliseconds)
    pub const SEARCH_DEBOUNCE_MS: u64 = 300;
    
    /// Auto-logout after inactivity (minutes)
    pub const AUTO_LOGOUT_MINUTES: u64 = 30;
}

/// Message/Chat related constants  
pub struct ChatConstants;

impl ChatConstants {
    /// Maximum message length
    pub const MAX_MESSAGE_LENGTH: usize = 1000;
    
    /// Maximum number of messages to cache per group
    pub const MAX_CACHED_MESSAGES_PER_GROUP: usize = 100;
    
    /// Message history batch size for loading
    pub const MESSAGE_BATCH_SIZE: u32 = 50;
    
    /// Real-time connection retry interval (seconds)
    pub const WEBSOCKET_RETRY_INTERVAL_SECONDS: u64 = 5;
}

/// Group management constants
pub struct GroupConstants;

impl GroupConstants {
    /// Maximum group name length
    pub const MAX_GROUP_NAME_LENGTH: usize = 256;
    
    /// Maximum group description length
    pub const MAX_GROUP_DESCRIPTION_LENGTH: usize = 1024;
    
    /// Maximum members per group
    pub const MAX_GROUP_MEMBERS: u32 = 100;
    
    /// Maximum groups a user can be in
    pub const MAX_USER_GROUPS: u32 = 50;
}

/// Performance monitoring constants (FR5)
pub struct MonitoringConstants;

impl MonitoringConstants {
    /// CPU usage logging interval (minutes)
    pub const CPU_LOG_INTERVAL_MINUTES: u64 = 2;
    
    /// Maximum log entries to keep in memory
    pub const MAX_LOG_ENTRIES: usize = 1000;
    
    /// Performance threshold for warnings (CPU %)
    pub const CPU_WARNING_THRESHOLD: f32 = 80.0;
    
    /// Memory threshold for warnings (MB)
    pub const MEMORY_WARNING_THRESHOLD_MB: u64 = 500;
}
