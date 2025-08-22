// API Services - Handle backend communication
pub mod auth;
pub mod user;
pub mod group;
pub mod invitation;

// Re-exports
pub use auth::AuthService;
pub use user::{UserService, UserSearchResult, UserProfileUpdate};
pub use group::{GroupChatService, GroupUpdateRequest};
pub use invitation::{InvitationService};
