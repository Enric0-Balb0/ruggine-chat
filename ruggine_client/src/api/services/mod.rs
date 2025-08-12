// API Services - Handle backend communication
pub mod auth;
pub mod user;

// Re-exports
pub use auth::AuthService;
pub use user::{UserService, UserSearchResult, UserProfileUpdate};
