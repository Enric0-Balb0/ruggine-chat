use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// =============================================================================
// ENUMS - Server synchronized
// =============================================================================

/// User gender from server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    #[serde(rename = "male")]
    Male,
    #[serde(rename = "female")]
    Female,
    #[serde(rename = "other")]
    Other,
}

/// User status from server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "suspended")]
    Suspended,
    #[serde(rename = "deleted")]
    Deleted,
    #[serde(rename = "banned")]
    Banned,
}

/// User type from server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserType {
    #[serde(rename = "end_user")]
    EndUser,
    #[serde(rename = "developer")]
    Developer,
    #[serde(rename = "admin")]
    Admin,
}

/// Current user action from server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurrentAction {
    #[serde(rename = "waiting")]
    Waiting,
    #[serde(rename = "writing")]
    Writing,
}

// =============================================================================
// DTOs - Server synchronized
// =============================================================================

/// User registration request - exact server DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRegisterRequest {
    pub address: String,
    pub birthday: NaiveDate,
    pub email: String,
    pub first_name: String,
    pub gender: serde_json::Value,  // Server uses serde_json::Value for enums
    pub last_name: String,
    pub password: String,
    pub username: String,
}

/// Profile update request - exact server DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdateRequest {
    pub address: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub first_name: Option<String>,
    pub gender: Option<serde_json::Value>,  // Server uses serde_json::Value for enums
    pub last_name: Option<String>,
}

/// User response wrapper from server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSuccessResponseUserReadDto {
    pub data: serde_json::Value,
}

/// Complete user profile with utility methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserProfile {
    pub id: i32,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub birthday: NaiveDate,
    pub address: String,
    pub gender: Gender,
    pub user_type: UserType,
    pub user_status: UserStatus,
    pub current_action: CurrentAction,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

/// Password change request - client specific
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

// =============================================================================
// CONVERSIONS - From server wrapper types
// =============================================================================

impl From<ApiSuccessResponseUserReadDto> for UserProfile {
    fn from(response: ApiSuccessResponseUserReadDto) -> Self {
        // TODO: Implement conversion when exact server structure is available
        todo!("Implement conversion from ApiSuccessResponseUserReadDto")
    }
}

// =============================================================================
// IMPLEMENTATIONS - Business logic methods
// =============================================================================

impl UserProfile {
    /// Get full name combining first and last name
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    /// Check if user is active
    pub fn is_active(&self) -> bool {
        self.user_status == UserStatus::Active
    }

    /// Check if user is online
    pub fn is_online(&self) -> bool {
        matches!(self.current_action, CurrentAction::Writing | CurrentAction::Waiting)
    }
    
    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.user_type == UserType::Admin
    }
    
    /// Check if user is developer
    pub fn is_developer(&self) -> bool {
        self.user_type == UserType::Developer
    }
    
    /// Check if user is end user
    pub fn is_end_user(&self) -> bool {
        self.user_type == UserType::EndUser
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, NaiveDate};

    #[test]
    fn test_user_profile_business_logic() {
        let user = UserProfile {
            id: 1,
            email: "test@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Test St".to_string(),
            gender: Gender::Male,
            user_type: UserType::EndUser,
            user_status: UserStatus::Active,
            current_action: CurrentAction::Waiting,
            created_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            updated_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            last_login: None,
        };

        // Test business logic methods
        assert_eq!(user.full_name(), "John Doe");
        assert!(user.is_active());
        assert!(user.is_online());
        assert!(!user.is_admin());
        assert!(!user.is_developer());
        assert!(user.is_end_user());
    }

    #[test]
    fn test_user_type_variants() {
        let admin = UserType::Admin;
        let developer = UserType::Developer;
        let end_user = UserType::EndUser;
        
        assert_ne!(admin, developer);
        assert_ne!(admin, end_user);
        assert_ne!(developer, end_user);
    }

    #[test]
    fn test_user_register_request_creation() {
        let request = UserRegisterRequest {
            address: "123 Test St".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            email: "test@example.com".to_string(),
            first_name: "John".to_string(),
            gender: serde_json::json!("male"),
            last_name: "Doe".to_string(),
            password: "securepass123".to_string(),
            username: "johndoe".to_string(),
        };

        assert!(!request.email.is_empty());
        assert!(request.email.contains('@'));
        assert!(!request.password.is_empty());
        assert!(!request.first_name.is_empty());
        assert!(!request.last_name.is_empty());
    }
}
