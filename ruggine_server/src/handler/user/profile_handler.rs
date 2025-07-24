use crate::dto::user_dto::UserReadDto;
use crate::entity::user::User;
use crate::response::api_response::ApiSuccessResponse;
use axum::{Extension, Json};

#[utoipa::path(
    get,
    path = "/api/user/profile",
    responses(
        (status = 200, description = "User profile retrieved successfully", body = UserReadDto),
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
    use crate::entity::user::User;
    use crate::factory::user_factory::UserFactory;
    use axum::Extension;

    #[tokio::test]
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

    #[tokio::test]
    async fn test_profile_with_inactive_user() {
        // Arrange: create an inactive user using the factory
        let mut user = UserFactory::fake_user();
        user.is_active = 0; // Set user as inactive
        user.id = 2;
        user.email = "inactive@example.com".to_string();

        // Act: call the profile handler
        let response = profile(Extension(user.clone())).await;

        // Assert: verify the response contains the inactive user data
        let data = response.0.data();
        let expected_dto = UserReadDto::from(user);

        assert_eq!(*data, expected_dto);
        assert_eq!(data.is_active, 0);
    }

    #[tokio::test]
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
            updated_at: Some(chrono::Utc::now()),
            is_active: 1,
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
        assert_eq!(data.is_active, user.is_active);
    }

    #[tokio::test]
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
}
