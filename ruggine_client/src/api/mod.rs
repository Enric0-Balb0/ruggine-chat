// API module - Backend communication layer
pub mod client;
pub mod error;
pub mod services;
pub mod facade;

// Re-exports
pub use client::ApiClient;
pub use error::HttpError;
pub use services::{AuthService, UserService, UserSearchResult, UserProfileUpdate};
pub use facade::RuggineApiClient;
