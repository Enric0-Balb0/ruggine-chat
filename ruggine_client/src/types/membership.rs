use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use super::invitation::{MemberRole, MembershipStatus};

// =============================================================================
// ADDITIONAL ENUMS (from OpenAPI)
// =============================================================================

/// Current action for membership as defined in OpenAPI ("waiting" | "writing")
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurrentAction {
    Waiting,
    Writing,
}

// =============================================================================
// DTOs - Server synchronized
// =============================================================================

/// Group membership response wrapper from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSuccessResponseGroupMembershipReadDto {
    pub data: GroupMembershipReadDto,
}

/// Vector of group memberships response wrapper from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSuccessResponseVecGroupMembershipReadDto {
    pub data: Vec<GroupMembershipReadDto>,
}

/// Group membership data from server - exact structure from OpenAPI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMembershipReadDto {
    pub id: i32,
    pub user_id: i32,
    pub group_chat_id: i32,
    pub role: MemberRole,
    pub joined_at: String, // date-time format from OpenAPI
    pub left_at: Option<String>, // date-time format, nullable
    pub membership_status: MembershipStatus,
    pub invitation_id: i32,
    pub current_action: CurrentAction,
}

// =============================================================================
// CLIENT TYPES - Rich types with business logic
// =============================================================================

/// Complete group membership with utility methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupMembership {
    pub id: i32,
    pub user_id: i32,
    pub group_chat_id: i32,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub membership_status: MembershipStatus,
    pub invitation_id: i32,
    pub current_action: CurrentAction,
    
    // Denormalized data for UI
    pub group_name: Option<String>,
    pub user_name: Option<String>,
}

// =============================================================================
// CONVERSIONS - From server wrapper types
// =============================================================================

impl From<ApiSuccessResponseGroupMembershipReadDto> for GroupMembership {
    fn from(response: ApiSuccessResponseGroupMembershipReadDto) -> Self {
        let membership_data = response.data;
        
        Self {
            id: membership_data.id,
            user_id: membership_data.user_id,
            group_chat_id: membership_data.group_chat_id,
            role: membership_data.role,
            joined_at: membership_data.joined_at.parse().unwrap_or_default(),
            left_at: membership_data.left_at
                .as_ref()
                .and_then(|s| s.parse().ok()),
            membership_status: membership_data.membership_status,
            invitation_id: membership_data.invitation_id,
            current_action: membership_data.current_action,
            
            // Denormalized data for UI - not provided by server
            group_name: None,
            user_name: None,
        }
    }
}

impl From<ApiSuccessResponseVecGroupMembershipReadDto> for Vec<GroupMembership> {
    fn from(response: ApiSuccessResponseVecGroupMembershipReadDto) -> Self {
        response.data.into_iter().map(|membership_data| {
            GroupMembership {
                id: membership_data.id,
                user_id: membership_data.user_id,
                group_chat_id: membership_data.group_chat_id,
                role: membership_data.role,
                joined_at: membership_data.joined_at.parse().unwrap_or_default(),
                left_at: membership_data.left_at
                    .as_ref()
                    .and_then(|s| s.parse().ok()),
                membership_status: membership_data.membership_status,
                invitation_id: membership_data.invitation_id,
                current_action: membership_data.current_action,
                
                // Denormalized data for UI - not provided by server
                group_name: None,
                user_name: None,
            }
        }).collect()
    }
}

// =============================================================================
// IMPLEMENTATIONS - Business logic methods
// =============================================================================

impl GroupMembership {
    /// Check if membership is active
    pub fn is_active(&self) -> bool {
        self.membership_status == MembershipStatus::Active
    }
    
    /// Check if user is admin of the group
    pub fn is_admin(&self) -> bool {
        self.role == MemberRole::Admin
    }
    
    /// Check if user is a regular member
    pub fn is_member(&self) -> bool {
        self.role == MemberRole::Member
    }
    
    /// Check if user has left the group
    pub fn has_left(&self) -> bool {
        self.membership_status == MembershipStatus::Left
    }
    
    /// Check if user is banned from the group
    pub fn is_banned(&self) -> bool {
        self.membership_status == MembershipStatus::Banned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_membership_business_logic() {
        let membership = GroupMembership {
            id: 1,
            user_id: 1,
            group_chat_id: 1,
            role: MemberRole::Admin,
            joined_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            left_at: None,
            membership_status: MembershipStatus::Active,
            invitation_id: 1,
            current_action: CurrentAction::Waiting,
            group_name: None,
            user_name: None,
        };

        assert!(membership.is_active());
        assert!(membership.is_admin());
        assert!(!membership.is_member());
        assert!(!membership.has_left());
        assert!(!membership.is_banned());
    }

    #[test]
    fn test_member_role_variants() {
        let admin = MemberRole::Admin;
        let member = MemberRole::Member;
        
        assert_ne!(admin, member);
    }

    #[test]
    fn test_membership_status_variants() {
        let active = MembershipStatus::Active;
        let left = MembershipStatus::Left;
        let banned = MembershipStatus::Banned;
        
        assert_ne!(active, left);
        assert_ne!(active, banned);
        assert_ne!(left, banned);
    }
}
