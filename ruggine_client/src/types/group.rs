use serde::{Deserialize, Serialize};

/// Group chat creation request - exact server DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupChatCreateRequest {
    pub description: String,
    pub name: String,
}

/// Group chat response wrapper from server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSuccessResponseGroupChatReadDto {
    pub data: serde_json::Value,
}

// =============================================================================
// CLIENT TYPES - Rich types with business logic
// =============================================================================

/// Complete group chat with utility methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChat {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub created_by: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub member_count: Option<i32>,
    pub is_active: bool,
}

// =============================================================================
// CONVERSIONS - From server wrapper types
// =============================================================================

impl From<ApiSuccessResponseGroupChatReadDto> for GroupChat {
    fn from(response: ApiSuccessResponseGroupChatReadDto) -> Self {
        // TODO: Implement conversion when exact server structure is available
        todo!("Implement conversion from ApiSuccessResponseGroupChatReadDto")
    }
}

// =============================================================================
// IMPLEMENTATIONS - Business logic methods
// =============================================================================

impl GroupChat {
    /// Check if group is active
    pub fn is_active(&self) -> bool {
        self.is_active
    }
    
    /// Get short description for UI display
    pub fn short_description(&self) -> String {
        if self.description.len() > 50 {
            format!("{}...", &self.description[..50])
        } else {
            self.description.clone()
        }
    }
}
