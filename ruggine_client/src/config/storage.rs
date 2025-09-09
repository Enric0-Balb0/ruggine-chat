/// Storage keys and configuration
pub struct StorageKeys;

impl StorageKeys {
    // Authentication storage
    pub const AUTH_TOKEN: &'static str = "ruggine_auth_token";
    pub const USER_PROFILE: &'static str = "ruggine_user_profile";
    pub const LOGIN_ATTEMPTS: &'static str = "ruggine_login_attempts";
    pub const LAST_LOGIN: &'static str = "ruggine_last_login";
    
    // Remember Me functionality
    pub const REMEMBER_ME_ENABLED: &'static str = "ruggine_remember_me";
    pub const REMEMBER_ME_CREDENTIALS: &'static str = "ruggine_remember_credentials";
    pub const REMEMBER_ME_EXPIRY: &'static str = "ruggine_remember_expiry";
    
    // Chat storage
    pub const MESSAGES_PREFIX: &'static str = "ruggine_messages_";
    pub const GROUPS: &'static str = "ruggine_groups";
    pub const INVITATIONS: &'static str = "ruggine_invitations";
    pub const UNREAD_COUNTS: &'static str = "ruggine_unread_counts";
    
    // Application state
    pub const APP_SETTINGS: &'static str = "ruggine_app_settings";
    pub const THEME_PREFERENCE: &'static str = "ruggine_theme";
    pub const LANGUAGE_PREFERENCE: &'static str = "ruggine_language";
    
    // Performance monitoring (FR5)
    pub const CPU_USAGE_LOG: &'static str = "ruggine_cpu_log";
    pub const PERFORMANCE_METRICS: &'static str = "ruggine_performance";
    pub const LAST_CPU_CHECK: &'static str = "ruggine_last_cpu_check";
    
    // Cache management
    pub const CACHE_VERSION: &'static str = "ruggine_cache_version";
    pub const LAST_SYNC: &'static str = "ruggine_last_sync";
    
    // Utility methods for dynamic keys
    pub fn messages_for_group(group_id: &str) -> String {
        format!("{}{}", Self::MESSAGES_PREFIX, group_id)
    }
    
    pub fn user_setting(setting_name: &str) -> String {
        format!("ruggine_user_setting_{}", setting_name)
    }
    
    pub fn cache_key(entity: &str, id: &str) -> String {
        format!("ruggine_cache_{}_{}", entity, id)
    }
}

/// Storage configuration and limits
pub struct StorageConfig;

impl StorageConfig {
    /// Maximum items to store for each category
    pub const MAX_CACHED_MESSAGES_PER_GROUP: usize = 100;
    pub const MAX_CACHED_GROUPS: usize = 50;
    pub const MAX_CACHED_INVITATIONS: usize = 100;
    pub const MAX_LOG_ENTRIES: usize = 1000;
    
    /// Storage quotas (in KB)
    pub const MAX_STORAGE_SIZE_KB: usize = 10240; // 10MB
    pub const WARNING_STORAGE_SIZE_KB: usize = 8192; // 8MB
    
    /// Cache expiration times (in minutes)
    pub const CACHE_EXPIRY_MINUTES: u64 = 60;
    pub const OFFLINE_CACHE_EXPIRY_MINUTES: u64 = 1440; // 24 hours
    
    /// Remember Me settings
    pub const REMEMBER_ME_DURATION_DAYS: u64 = 30; // 30 giorni
    pub const AUTO_REFRESH_BEFORE_EXPIRY_HOURS: u64 = 2; // Refresh 2h prima della scadenza
    
    /// Cleanup intervals
    pub const CLEANUP_INTERVAL_MINUTES: u64 = 30;
    pub const PERFORMANCE_LOG_CLEANUP_DAYS: u64 = 7;
}
