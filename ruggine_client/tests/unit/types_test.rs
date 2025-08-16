// Unit tests for client types
// Testing type behavior, serialization, and business logic

use ruggine_client_ui::types::user::{UserProfile, UserType, UserStatus, CurrentAction, Gender, UserRegisterRequest};
use ruggine_client_ui::types::auth::{LoginRequest, TokenResponse};
use ruggine_client_ui::types::common::LoadingState;
use chrono::{DateTime, NaiveDate};

#[cfg(test)]
mod user_types_tests {
    use super::*;

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
            is_online: true,  // Add missing field
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
    fn test_user_status_variants() {
        let statuses = vec![
            UserStatus::Pending,
            UserStatus::Active,
            UserStatus::Suspended,
            UserStatus::Deleted,
            UserStatus::Banned,
        ];
        
        for status in &statuses {
            assert!(matches!(status, 
                UserStatus::Pending | 
                UserStatus::Active | 
                UserStatus::Suspended | 
                UserStatus::Deleted | 
                UserStatus::Banned
            ));
        }
    }

    #[test]
    fn test_user_register_request_creation() {
        let request = UserRegisterRequest {
            address: "123 Test St".to_string(),
            birthday: "1990-01-01".to_string(), // String format
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
        assert_eq!(request.birthday, "1990-01-01");
        assert_eq!(request.gender, Gender::Male);
    }
}

#[cfg(test)]
mod auth_types_tests {
    use super::*;

    #[test]
    fn test_login_request_creation() {
        let request = LoginRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        assert!(!request.email.is_empty());
        assert!(request.email.contains('@'));
        assert!(!request.password.is_empty());
    }

    #[test]
    fn test_token_response_business_logic() {
        let now = chrono::Utc::now().timestamp();
        
        // Valid token
        let valid_token = TokenResponse {
            token: "valid_token".to_string(),
            iat: now,
            exp: now + 3600, // 1 hour from now
        };

        // Expired token
        let expired_token = TokenResponse {
            token: "expired_token".to_string(),
            iat: now - 7200, // 2 hours ago
            exp: now - 3600, // 1 hour ago
        };

        assert!(!valid_token.is_expired());
        assert!(expired_token.is_expired());
        
        assert!(valid_token.time_to_expiry() > 0);
        assert_eq!(expired_token.time_to_expiry(), 0);
    }
}

#[cfg(test)]
mod common_types_tests {
    use super::*;

    #[test]
    fn test_loading_state_behavior() {
        let idle: LoadingState<String> = LoadingState::Idle;
        let loading: LoadingState<String> = LoadingState::Loading;
        let success = LoadingState::Success("data".to_string());
        let error: LoadingState<String> = LoadingState::Error("error".to_string());

        assert!(!idle.is_loading());
        assert!(loading.is_loading());
        assert!(!success.is_loading());
        assert!(!error.is_loading());

        assert!(!idle.is_success());
        assert!(!loading.is_success());
        assert!(success.is_success());
        assert!(!error.is_success());

        assert!(!idle.is_error());
        assert!(!loading.is_error());
        assert!(!success.is_error());
        assert!(error.is_error());

        assert!(idle.data().is_none());
        assert!(loading.data().is_none());
        assert_eq!(success.data(), Some(&"data".to_string()));
        assert!(error.data().is_none());

        assert!(idle.error().is_none());
        assert!(loading.error().is_none());
        assert!(success.error().is_none());
        assert_eq!(error.error(), Some(&"error".to_string()));
    }

    #[test]
    fn test_loading_state_default() {
        let default_state: LoadingState<i32> = LoadingState::default();
        assert!(matches!(default_state, LoadingState::Idle));
    }
}
