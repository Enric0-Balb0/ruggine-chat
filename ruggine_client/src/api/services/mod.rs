pub mod message;
// API Services - Handle backend communication
pub mod auth;
pub mod user;
pub mod group;
pub mod invitation;
pub mod membership;

// Re-exports
pub use auth::AuthService;
pub use user::{UserService, UserSearchResult, UserProfileUpdate};
pub use group::{GroupChatService, GroupUpdateRequest};
pub use membership::GroupMembershipService;
pub use invitation::{InvitationService};
