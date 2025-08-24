// API module - Backend communication layer
pub mod client;
pub mod http_error;
pub mod services;
pub mod facade;
pub mod ws;

// Re-exports
pub use client::ApiClient;
pub use http_error::HttpError;
pub use services::{AuthService, UserService, UserSearchResult, UserProfileUpdate};
pub use facade::RuggineApiClient;
pub use ws::message_ws_service::MessageWsService;
