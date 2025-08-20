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

/// User registration request - exact server DTO (UserRegisterDto)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisterRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub birthday: String, // date format as per OpenAPI
    pub address: String,
    pub gender: Gender,
}

/// Profile update request - exact server DTO (ProfileUpdateDto)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdateRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub birthday: Option<String>, // date format as per OpenAPI
    pub address: Option<String>,
    pub gender: Option<Gender>,
}

/// User response wrapper from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSuccessResponseUserReadDto {
    pub data: UserReadDto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReadDto {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub created_at: String,
    pub updated_at: String,
    pub user_status: UserStatus,
    pub user_type: UserType,
    pub birthday: String,
    pub is_online: bool,
    pub address: String,
    pub current_action: CurrentAction,
    pub gender: Gender,
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
    pub is_online: bool, // MISSING FIELD! From server UserReadDto
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
        let user_data = response.data;
        
        Self {
            id: user_data.id,
            email: user_data.email,
            first_name: user_data.first_name,
            last_name: user_data.last_name,
            username: user_data.username,
            birthday: user_data.birthday.parse().unwrap_or_default(), // Convert from string
            address: user_data.address,
            gender: user_data.gender,
            user_type: user_data.user_type,
            user_status: user_data.user_status,
            current_action: user_data.current_action,
            is_online: user_data.is_online, // Now correctly mapped from server
            created_at: user_data.created_at.parse().unwrap_or_default(), // Convert from string
            updated_at: user_data.updated_at.parse().unwrap_or_default(), // Convert from string
            last_login: None, // Not provided by server
        }
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

    /// Get personalized welcome message based on gender
    pub fn welcome_message(&self) -> String {
        match self.gender {
            Gender::Male => format!("Benvenuto, {}!", self.first_name),
            Gender::Female => format!("Benvenuta, {}!", self.first_name),
            Gender::Other => format!("Benvenuto/a, {}!", self.first_name),
        }
    }

    /// Get personalized welcome message for successful login/registration
    pub fn welcome_message_success(&self) -> String {
        match self.gender {
            Gender::Male => format!("Benvenuto {}!", self.first_name),
            Gender::Female => format!("Benvenuta {}!", self.first_name),
            Gender::Other => format!("Benvenuto/a {}!", self.first_name),
        }
    }

    /// Check if user is active
    pub fn is_active(&self) -> bool {
        self.user_status == UserStatus::Active
    }

    /// Check if user is online (based on server data)
    pub fn is_online(&self) -> bool {
        self.is_online // Now uses the actual server field
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
            is_online: true, // Added missing field
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
    fn test_user_profile_welcome_messages() {
        // Test male user
        let male_user = UserProfile {
            id: 1,
            email: "john@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Test St".to_string(),
            gender: Gender::Male,
            user_type: UserType::EndUser,
            user_status: UserStatus::Active,
            current_action: CurrentAction::Waiting,
            is_online: true,
            created_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            updated_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            last_login: None,
        };
        
        assert_eq!(male_user.welcome_message(), "Benvenuto, John!");
        assert_eq!(male_user.welcome_message_success(), "Benvenuto John!");

        // Test female user
        let female_user = UserProfile {
            id: 2,
            email: "jane@example.com".to_string(),
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            username: "janedoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "456 Test St".to_string(),
            gender: Gender::Female,
            user_type: UserType::EndUser,
            user_status: UserStatus::Active,
            current_action: CurrentAction::Waiting,
            is_online: true,
            created_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            updated_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            last_login: None,
        };
        
        assert_eq!(female_user.welcome_message(), "Benvenuta, Jane!");
        assert_eq!(female_user.welcome_message_success(), "Benvenuta Jane!");

        // Test other gender user
        let other_user = UserProfile {
            id: 3,
            email: "alex@example.com".to_string(),
            first_name: "Alex".to_string(),
            last_name: "Smith".to_string(),
            username: "alexsmith".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "789 Test St".to_string(),
            gender: Gender::Other,
            user_type: UserType::EndUser,
            user_status: UserStatus::Active,
            current_action: CurrentAction::Waiting,
            is_online: true,
            created_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            updated_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            last_login: None,
        };
        
        assert_eq!(other_user.welcome_message(), "Benvenuto/a, Alex!");
        assert_eq!(other_user.welcome_message_success(), "Benvenuto/a Alex!");
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
            birthday: "1990-01-01".to_string(),
            email: "test@example.com".to_string(),
            first_name: "John".to_string(),
            gender: Gender::Male,
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
