// =============================================================================
// DOMAIN MODULES - Organized by business domain
// =============================================================================

/// Common types and utilities
pub mod common;

/// Authentication types and utilities
pub mod auth;

/// User types and utilities
pub mod user;

/// Group chat types and utilities
pub mod group;

/// Invitation types and utilities
pub mod invitation;

/// Group membership types and utilities
pub mod membership;

// =============================================================================
// RE-EXPORTS - Public API
// =============================================================================

// Auth types
pub use auth::{LoginRequest, TokenResponse, TokenClaims, ApiSuccessResponseTokenReadDto, TokenReadDto};

// User types  
pub use user::{
    UserProfile, UserRegisterRequest, UserUpdateRequest, 
    ChangePasswordRequest, UserStatus, UserType, Gender, CurrentAction,
    ApiSuccessResponseUserReadDto, UserReadDto
};

// Group types
pub use group::{GroupChat, GroupChatCreateRequest, ApiSuccessResponseGroupChatReadDto, GroupChatReadDto};

// Invitation types
pub use invitation::{
    Invitation, InvitationCreateRequest, InvitationUpdateRequest,
    InvitationStatus, ApiSuccessResponseInvitationReadDto, InvitationReadDto, InvitationUpdateDto,
    MemberRole, MembershipStatus
};// Membership types
pub use membership::{
    GroupMembership, ApiSuccessResponseGroupMembershipReadDto,
    ApiSuccessResponseVecGroupMembershipReadDto, GroupMembershipReadDto
};

// Common types
pub use common::{
    ApiResponse, ApiSuccessResponseInvitationCreateDto, 
    ApiSuccessResponseInvitationUpdateDto
};
