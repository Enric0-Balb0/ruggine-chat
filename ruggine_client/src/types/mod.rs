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

/// Cpu usage log types
pub mod cpu_usage_log;

/// WebSocket message types
pub mod message_ws;

// =============================================================================
// RE-EXPORTS - Public API
// =============================================================================

// Auth types
pub use auth::TokenResponse;

// User types  
pub use user::UserProfile;

// Group types

// Invitation types
pub use invitation::{
    Invitation, InvitationCreateRequest, InvitationUpdateRequest, ApiSuccessResponseInvitationReadDto, ApiSuccessResponseVecInvitationReadDto, InvitationUpdateDto
};

// Membership types


// Message types

pub use message::*;

// WebSocket message types
pub use crate::types::message_ws::{
    WebSocketMessage, ClientAction, GroupAction
};

// Common types
pub use common::ApiSuccessResponseInvitationUpdateDto;

// Cpu usage types
pub use cpu_usage_log::{CpuUsageLogReadDto, PaginatedCpuUsageLogResponse, CpuUsagePage};
