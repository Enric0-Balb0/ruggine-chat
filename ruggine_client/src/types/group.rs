use serde::{Deserialize, Serialize};

/// Group chat creation request - exact server DTO (GroupChatCreateDto)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatCreateRequest {
    pub name: String,
    pub description: String,
}

/// Group chat response wrapper from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSuccessResponseGroupChatReadDto {
    pub data: GroupChatReadDto,
}

/// Vector of group chats response wrapper from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSuccessResponseVecGroupChatReadDto {
    pub data: Vec<GroupChatReadDto>,
}

/// Group chat data from server - exact structure from OpenAPI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChatReadDto {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub created_by: i32,
    pub created_at: String, // date-time format from OpenAPI
    pub updated_at: String, // date-time format from OpenAPI
}

// =============================================================================
// CLIENT TYPES - Rich types with business logic
// =============================================================================

/// Complete group chat with utility methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        let group_data = response.data;
        
        Self {
            id: group_data.id,
            name: group_data.name,
            description: group_data.description,
            created_by: group_data.created_by,
            created_at: group_data.created_at.parse().unwrap_or_default(), // Convert from string
            updated_at: group_data.updated_at.parse().unwrap_or_default(), // Convert from string
            member_count: None, // Not provided by server, client-side enhancement
            is_active: true, // Default to true, client-side enhancement
        }
    }
}

impl From<ApiSuccessResponseVecGroupChatReadDto> for Vec<GroupChat> {
    fn from(response: ApiSuccessResponseVecGroupChatReadDto) -> Self {
        response.data.into_iter().map(|group_data| {
            GroupChat {
                id: group_data.id,
                name: group_data.name,
                description: group_data.description,
                created_by: group_data.created_by,
                created_at: group_data.created_at.parse().unwrap_or_default(),
                updated_at: group_data.updated_at.parse().unwrap_or_default(),
                member_count: None,
                is_active: true,
            }
        }).collect()
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
