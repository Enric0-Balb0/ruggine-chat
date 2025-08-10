// Services module - Business logic layer
// Orchestrates API calls and manages application state

pub mod auth_service;
pub mod storage_service;
pub mod user_service;
pub mod theme_service;

// Re-export services for convenience
pub use auth_service::AuthService;
pub use storage_service::StorageService;
pub use user_service::{UserService, UserSearchResult, UserProfileUpdate};
pub use theme_service::{ThemeService, Theme, ThemeContext, ThemeProvider, use_theme};
