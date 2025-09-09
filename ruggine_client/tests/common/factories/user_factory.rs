// User-related test data factory

use super::base_factory::BaseFactory;
use chrono::{NaiveDate, DateTime, Utc};
use ruggine_client_ui::types::user::{
    UserRegisterRequest, UserProfile, UserStatus, UserType, Gender
};
use ruggine_client_ui::types::membership::CurrentAction;

pub struct UserFactory;

impl UserFactory {
    /// Create unique UserRegisterRequest for testing
    pub fn unique_user_register_request(prefix: &str) -> UserRegisterRequest {
        let (email, username, _) = BaseFactory::get_unique_user_info(prefix);
        
        UserRegisterRequest {
            email,
            password: "testpass123".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            username,
            birthday: "1990-01-01".to_string(),
            address: format!("Test Address {}", BaseFactory::get_unique_id()),
            gender: Gender::Other,
        }
    }
    
    /// Create basic user register request
    pub fn basic_user_register_request() -> UserRegisterRequest {
        UserRegisterRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: "1985-05-15".to_string(),
            address: "123 Test Street".to_string(),
            gender: Gender::Male,
        }
    }
    
    /// Create mock user profile for testing
    pub fn mock_user_profile() -> UserProfile {
        let unique_id = BaseFactory::get_unique_id();
        let (email, username, _) = BaseFactory::get_unique_user_info("mock");
        
        UserProfile {
            id: unique_id as i32,
            email,
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            username,
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "Test Address".to_string(),
            gender: Gender::Other,
            user_type: UserType::EndUser,
            user_status: UserStatus::Active,
            current_action: CurrentAction::Waiting,
            is_online: true,
            created_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            updated_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            last_login: None,
        }
    }
    
    /// Create admin user profile
    pub fn admin_user_profile() -> UserProfile {
        let mut profile = Self::mock_user_profile();
        profile.user_type = UserType::Admin;
        profile.username = "admin_user".to_string();
        profile.email = "admin@test.com".to_string();
        profile
    }
    
    /// Create inactive user profile
    pub fn inactive_user_profile() -> UserProfile {
        let mut profile = Self::mock_user_profile();
        profile.user_status = UserStatus::Suspended;
        profile.username = "inactive_user".to_string();
        profile.email = "inactive@test.com".to_string();
        profile
    }
    
    /// Create user with specific details
    pub fn user_with_details(id: i32, username: &str, email: &str) -> UserProfile {
        UserProfile {
            id,
            email: email.to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            username: username.to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "Test Address".to_string(),
            gender: Gender::Other,
            user_type: UserType::EndUser,
            user_status: UserStatus::Active,
            current_action: CurrentAction::Waiting,
            is_online: true,
            created_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            updated_at: DateTime::from_timestamp(1000000000, 0).unwrap(),
            last_login: None,
        }
    }

    /// Create user profile with specific first and last names for avatar tests
    pub fn user_profile_with_names(first_name: &str, last_name: &str) -> UserProfile {
        let mut profile = Self::mock_user_profile();
        profile.first_name = first_name.to_string();
        profile.last_name = last_name.to_string();
        profile
    }

    /// Create unique user info (email, username, password) with prefix
    pub fn get_unique_user_info(prefix: &str) -> (String, String, String) {
        BaseFactory::get_unique_user_info(prefix)
    }
}
