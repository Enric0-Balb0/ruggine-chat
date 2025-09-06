/// Utility functions module

// Client-side services
pub mod storage;
pub mod theme;
pub mod error_messages;
pub mod timers;
pub mod error_recovery;

// Re-exports for services
pub use storage::StorageService;
pub use theme::{Theme, ThemeProvider, use_theme};
pub use error_messages::{auth_error_to_login_message, auth_error_to_register_message};
pub use timers::sleep_ms;
pub use error_recovery::{ErrorRecovery, RetryConfig, NetworkOperation};

/// Common validation utilities
pub mod validation {
    /// Validate email format
    pub fn is_valid_email(email: &str) -> bool {
        email.contains('@') && email.len() > 3
    }

    /// Validate password strength
    pub fn is_valid_password(password: &str) -> bool {
        password.len() >= 8
    }
}

/// Common string utilities
pub mod string {
    /// Truncate string to max length
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }
}
