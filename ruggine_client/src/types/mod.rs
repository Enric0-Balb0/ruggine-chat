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

/// Message types and utilities
pub mod message;

/// WebSocket message types
pub mod message_ws;

// =============================================================================
// RE-EXPORTS - Public API
// =============================================================================

// Auth types
pub use auth::{LoginRequest, TokenResponse, TokenClaims, ApiSuccessResponseTokenReadDto, TokenReadDto};

// User types  
pub use user::{
    UserProfile, UserRegisterRequest, UserUpdateRequest, 
    ChangePasswordRequest, UserStatus, UserType, Gender,
    ApiSuccessResponseUserReadDto, UserReadDto
};

// Group types
pub use group::{GroupChat, GroupChatCreateRequest, ApiSuccessResponseGroupChatReadDto, GroupChatReadDto};

// Invitation types
pub use invitation::{
    Invitation, InvitationCreateRequest, InvitationUpdateRequest,
    InvitationStatus, ApiSuccessResponseInvitationReadDto, ApiSuccessResponseVecInvitationReadDto, InvitationReadDto, InvitationUpdateDto,
    MemberRole, MembershipStatus
};

// Membership types
pub use membership::{
    GroupMembership, ApiSuccessResponseGroupMembershipReadDto,
    ApiSuccessResponseVecGroupMembershipReadDto, GroupMembershipReadDto
};


// Message types
pub use message::{
    TextMessageCreateRequest, TextMessageReadDto, PaginatedTextMessageResponse, PaginationMetadataDto,
    Message, MessagePage, PaginationMetadata
};

// WebSocket message types
pub use crate::types::message_ws::{
    WebSocketMessage, ClientAction, GroupAction, ServerEvent, GroupEvent, NotificationEvent, ControlMessage, WsError
};

// Common types
pub use common::{
    ApiResponse, ApiSuccessResponseInvitationCreateDto, 
    ApiSuccessResponseInvitationUpdateDto
};
