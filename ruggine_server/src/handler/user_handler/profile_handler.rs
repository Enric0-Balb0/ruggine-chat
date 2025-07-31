use crate::dto::user_dto::UserReadDto;
use crate::entity::user::User;
use crate::response::api_response::ApiSuccessResponse;
use axum::{Extension, Json};

#[utoipa::path(
    get,
    path = "/api/user/profile",
    responses(
        (status = 200, description = "User profile retrieved successfully", body = ApiSuccessResponseUserReadDto),
        (status = 401, description = "Unauthorized - Invalid or missing token")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "User"
)]
pub async fn profile(
    Extension(current_user): Extension<User>,
) -> Json<ApiSuccessResponse<UserReadDto>> {
    Json(ApiSuccessResponse::send(UserReadDto::from(current_user)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::user_dto::UserReadDto;
    use crate::entity::user::{User, UserStatus};
    use crate::factory::user_factory::UserFactory;
    use axum::Extension;

    #[tokio_shared_rt::test(shared)]
    async fn test_profile_returns_expected_user_dto() {
        // Arrange: create a mock user
        let user = UserFactory::fake_user();

        // Act: call the profile handler
        let response = profile(Extension(user.clone())).await;

        // Assert: unpack and check the response
        let data = response.0.data();

        // You can use From or manually construct expected dto
        let expected_dto = UserReadDto::from(user);

        assert_eq!(*data, expected_dto);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_profile_with_inactive_user() {
        // Arrange: create an inactive user using the factory
        let mut user = UserFactory::fake_user();
        user.user_status = UserStatus::Deleted; // Set user as inactive
        user.id = 2;
        user.email = "inactive@example.com".to_string();

        // Act: call the profile handler
        let response = profile(Extension(user.clone())).await;

        // Assert: verify the response contains the inactive user data
        let data = response.0.data();
        let expected_dto = UserReadDto::from(user);

        assert_eq!(*data, expected_dto);
        assert_eq!(data.user_status, UserStatus::Deleted);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_profile_preserves_all_user_fields() {
        // Arrange: create a user with specific field values
        let user = User {
            id: 42,
            first_name: "Jane".to_string(),
            last_name: "Smith".to_string(),
            username: "janesmith".to_string(),
            email: "jane.smith@test.com".to_string(),
            password: "secret_hash".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            user_status: Default::default(),
            user_type: Default::default(), // Default user type
            birthday: chrono::NaiveDate::from_ymd_opt(1992, 8, 20).unwrap(),
            is_online: true,
            address: "789 Profile St".to_string(),
            current_action: crate::entity::user::CurrentAction::Writing,
            gender: crate::entity::user::Gender::Female,
        };

        // Act: call the profile handler
        let response = profile(Extension(user.clone())).await;

        // Assert: verify all fields are correctly mapped (excluding password)
        let data = response.0.data();
        
        assert_eq!(data.id, user.id);
        assert_eq!(data.first_name, user.first_name);
        assert_eq!(data.last_name, user.last_name);
        assert_eq!(data.username, user.username);
        assert_eq!(data.email, user.email);
        assert_eq!(data.created_at, user.created_at);
        assert_eq!(data.updated_at, user.updated_at);
        assert_eq!(data.user_status, user.user_status);
        assert_eq!(data.birthday, user.birthday);
        assert_eq!(data.is_online, user.is_online);
        assert_eq!(data.address, user.address);
        assert_eq!(data.current_action, user.current_action);
        assert_eq!(data.gender, user.gender);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_profile_response_structure() {
        // Arrange: create a mock user
        let user = UserFactory::fake_user();

        // Act: call the profile handler
        let response = profile(Extension(user)).await;

        // Assert: verify the response structure is correct
        let json_response = response.0;
        let data = json_response.data();
        
        // Verify that the response contains a UserReadDto
        assert!(data.id > 0);
        assert!(!data.first_name.is_empty());
        assert!(!data.last_name.is_empty());
        assert!(!data.username.is_empty());
        assert!(data.email.contains('@'));
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_profile_includes_new_fields() {
        // Arrange: create a user with all new fields set
        let user = User {
            id: 100,
            first_name: "New".to_string(),
            last_name: "Fields".to_string(),
            username: "newfields".to_string(),
            email: "newfields@test.com".to_string(),
            password: "hash123".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            user_status: crate::entity::user::UserStatus::Active,
            user_type: crate::entity::user::UserType::Developer,
            birthday: chrono::NaiveDate::from_ymd_opt(1988, 3, 15).unwrap(),
            is_online: false,
            address: "321 New Field Blvd".to_string(),
            current_action: crate::entity::user::CurrentAction::Waiting,
            gender: crate::entity::user::Gender::Other,
        };

        // Act: call the profile handler
        let response = profile(Extension(user.clone())).await;

        // Assert: verify all new fields are present in the response
        let data = response.0.data();
        
        assert_eq!(data.birthday, user.birthday);
        assert_eq!(data.is_online, user.is_online);
        assert_eq!(data.address, user.address);
        assert_eq!(data.current_action, user.current_action);
        assert_eq!(data.gender, user.gender);
        assert_eq!(data.user_type, user.user_type);
    }
}
