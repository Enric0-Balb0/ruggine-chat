/// Utility functions module

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
