use std::fmt;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Member role from server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemberRole {
    #[serde(rename = "member")]
    Member,
    #[serde(rename = "admin")]
    Admin,
}

impl MemberRole {
    pub fn display_name(&self) -> &'static str {
        match self {
            MemberRole::Admin => "Admin",
            MemberRole::Member => "Membro",
        }
    }
}

/// Membership status from server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MembershipStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "banned")]
    Banned,
}

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

// Implementazione Display per InvitationStatus
impl fmt::Display for InvitationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            InvitationStatus::Pending => "pending",
            InvitationStatus::Accepted => "accepted",
            InvitationStatus::Rejected => "rejected",
        };
        write!(f, "{}", s)
    }
}

// =============================================================================
// DTOs - Server synchronized
// =============================================================================

/// Invitation creation request - exact server DTO (InvitationCreateDto)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationCreateRequest {
    pub to_user_id: i32,
    pub group_chat_id: i32,
    pub role_at_join: MemberRole, 
}

/// Invitation status update request - exact server DTO (InvitationUpdateStatusDto)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationUpdateRequest {
    pub status: InvitationStatus,
    pub invitation_id: i32, 
}

/// Invitation response wrapper from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSuccessResponseInvitationReadDto {
    pub data: InvitationReadDto,
}

/// Vector of group memberships response wrapper from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSuccessResponseVecInvitationReadDto {
    pub data: Vec<InvitationReadDto>,
}

/// Invitation data from server - exact structure from OpenAPI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationReadDto {
    pub id: i32,
    pub from_user_id: i32,
    pub to_user_id: i32,
    pub group_chat_id: i32,
    pub status: InvitationStatus,
    pub sent_at: String, // date-time format from OpenAPI
    pub responded_at: Option<String>, // date-time format, nullable
    pub role_at_join: MemberRole, 
}

/// Invitation update response data from server - from ApiSuccessResponseInvitationUpdateDto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationUpdateDto {
    pub id: i32,
    pub status: InvitationStatus,
    pub responded_at: String, // date-time format from OpenAPI
    pub group_membership_id: Option<i32>, // MISSING FIELD! Nullable from OpenAPI
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
    pub role_at_join: MemberRole,
    // Denormalized data for UI
    pub group_name: Option<String>,
    pub from_user_name: Option<String>,
    pub to_user_name: Option<String>,
}

// =============================================================================
// CONVERSIONS - From server wrapper types
// =============================================================================

impl From<ApiSuccessResponseInvitationReadDto> for Invitation {
    fn from(response: ApiSuccessResponseInvitationReadDto) -> Self {
        let invitation_data = response.data;
        Self {
            id: invitation_data.id,
            group_chat_id: invitation_data.group_chat_id,
            from_user_id: invitation_data.from_user_id,
            to_user_id: invitation_data.to_user_id,
            status: invitation_data.status,
            created_at: invitation_data.sent_at.parse().unwrap_or_default(),
            updated_at: invitation_data.responded_at
                .as_ref()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| invitation_data.sent_at.parse().unwrap_or_default()),
            expires_at: None,
            role_at_join: invitation_data.role_at_join,
            group_name: None,
            from_user_name: None,
            to_user_name: None,
        }
    }
}

impl From<ApiSuccessResponseVecInvitationReadDto> for Vec<Invitation> {
    fn from(response: ApiSuccessResponseVecInvitationReadDto) -> Self {
        response.data.into_iter().map(|invitation_data| {
            Invitation {
                id: invitation_data.id,
                group_chat_id: invitation_data.group_chat_id,
                from_user_id: invitation_data.from_user_id,
                to_user_id: invitation_data.to_user_id,
                status: invitation_data.status,
                created_at: invitation_data.sent_at.parse().unwrap_or_default(),
                updated_at: invitation_data.responded_at
                    .as_ref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| invitation_data.sent_at.parse().unwrap_or_default()),
                expires_at: None,
                role_at_join: invitation_data.role_at_join,
                group_name: None,
                from_user_name: None,
                to_user_name: None,
            }
        }).collect()
    }
}

/// =============================================================================
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
