use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Invitation status from server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvitationStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "accepted")]
    Accepted,
    #[serde(rename = "rejected")]
    Rejected,
}

// =============================================================================
// DTOs - Server synchronized
// =============================================================================

/// Invitation creation request - exact server DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationCreateRequest {
    pub group_chat_id: i32,
    pub to_user_id: i32,
}

/// Invitation status update request - exact server DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationUpdateRequest {
    pub status: serde_json::Value,  // Server uses serde_json::Value for enums
}

/// Invitation response wrapper from server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSuccessResponseInvitationReadDto {
    pub data: serde_json::Value,
}

// =============================================================================
// CLIENT TYPES - Rich types with business logic
// =============================================================================

/// Complete invitation with utility methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    pub id: i32,
    pub group_chat_id: i32,
    pub from_user_id: i32,
    pub to_user_id: i32,
    pub status: InvitationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    
    // Denormalized data for UI
    pub group_name: Option<String>,
    pub from_user_name: Option<String>,
    pub to_user_name: Option<String>,
}

// =============================================================================
// CONVERSIONS - From server wrapper types
// =============================================================================

impl From<ApiSuccessResponseInvitationReadDto> for Invitation {
    fn from(_response: ApiSuccessResponseInvitationReadDto) -> Self {
        // TODO: Implement conversion when exact server structure is available
        todo!("Implement conversion from ApiSuccessResponseInvitationReadDto")
    }
}

// =============================================================================
// IMPLEMENTATIONS - Business logic methods
// =============================================================================

impl Invitation {
    /// Check if invitation is still valid
    pub fn is_valid(&self) -> bool {
        self.status == InvitationStatus::Pending && 
        self.expires_at.map(|exp| exp > Utc::now()).unwrap_or(true)
    }
    
    /// Check if invitation is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| exp <= Utc::now()).unwrap_or(false)
    }
    
    /// Check if invitation is accepted
    pub fn is_accepted(&self) -> bool {
        self.status == InvitationStatus::Accepted
    }
    
    /// Check if invitation is rejected
    pub fn is_rejected(&self) -> bool {
        self.status == InvitationStatus::Rejected
    }
    
    /// Get user-friendly description
    pub fn description(&self) -> String {
        match &self.group_name {
            Some(group) => format!("Invitation to join '{}'", group),
            None => format!("Invitation to group ID {}", self.group_chat_id),
        }
    }
}
