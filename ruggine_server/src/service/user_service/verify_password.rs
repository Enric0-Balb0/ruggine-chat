use crate::entity::user::User;
use crate::service::user_service::UserService;

impl UserService {
    pub fn verify_password_internal(&self, user: &User, password: &str) -> bool {
        bcrypt::verify(password, &user.password).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::user::{User};
    use crate::repository::user_repository::user_repository_trait::MockUserRepositoryTrait;
    use chrono::{Utc, NaiveDate};
    use std::sync::Arc;

    #[test]
    fn test_verify_password_correct() {
        // Arrange: Create a user with a known password
        let password = "testpassword123";
        let hashed_password = bcrypt::hash(password, 4).unwrap();

        let user = User {
            id: 1,
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: hashed_password,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_status: Default::default(),
            user_type: Default::default(), // Default user type
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            is_online: false,
            address: "123 Test St".to_string(),
            current_action: Default::default(),
            gender: Default::default(),
        };

        let mock_repo = MockUserRepositoryTrait::new();
        let service = UserService::with_repo(Arc::new(mock_repo));

        // Act: Verify the correct password
        let result = service.verify_password_internal(&user, password);

        // Assert: Should return true for correct password
        assert!(result);
    }

    #[test]
    fn test_verify_password_incorrect() {
        // Arrange: Create a user with a known password
        let correct_password = "testpassword123";
        let incorrect_password = "wrongpassword";
        let hashed_password = bcrypt::hash(correct_password, 4).unwrap();

        let user = User {
            id: 1,
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: hashed_password,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_status: Default::default(),
            user_type: Default::default(), // Default user type
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            is_online: false,
            address: "123 Test St".to_string(),
            current_action: Default::default(),
            gender: Default::default(),
        };

        let mock_repo = MockUserRepositoryTrait::new();
        let service = UserService::with_repo(Arc::new(mock_repo));

        // Act: Verify an incorrect password
        let result = service.verify_password_internal(&user, incorrect_password);

        // Assert: Should return false for incorrect password
        assert!(!result);
    }

    #[test]
    fn test_verify_password_empty_password() {
        // Arrange: Create a user with a known password
        let correct_password = "testpassword123";
        let hashed_password = bcrypt::hash(correct_password, 4).unwrap();

        let user = User {
            id: 1,
            first_name: "Test".into(),
            last_name: "User".into(),
            username: "testuser".into(),
            email: "test@example.com".into(),
            password: hashed_password,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_status: Default::default(),
            user_type: Default::default(), // Default user type
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            is_online: false,
            address: "123 Test St".to_string(),
            current_action: Default::default(),
            gender: Default::default(),
        };

        let mock_repo = MockUserRepositoryTrait::new();
        let service = UserService::with_repo(Arc::new(mock_repo));

        // Act: Verify an empty password
        let result = service.verify_password_internal(&user, "");

        // Assert: Should return false for empty password
        assert!(!result);
    }

}