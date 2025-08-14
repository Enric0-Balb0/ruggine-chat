use crate::common::{cleanup_user_by_email, create_user_router};
use axum::http::{Method, StatusCode};
use ruggine_server::dto::user_dto::{ProfileUpdateDto};
use ruggine_server::entity::user::Gender;
use ruggine_server::factory::user_factory::UserFactory;
use serde_json::json;
use tower::ServiceExt;
use axum::body::Body;
use axum::http::Request;
use chrono::NaiveDate;
use axum::body::to_bytes;
use crate::common::{login_and_get_token, create_test_user};

#[cfg(test)]
mod update_profile_e2e_tests {
    use ruggine_server::entity::user::all_genders;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_success_complete() {
        // Arrange: Create user and get token
        let (user, password) = create_test_user("e2e_update_profile_success").await;
        let token = login_and_get_token(user.email.clone(), password.clone()).await;
        let app = create_user_router().await;

        let update_dto = ProfileUpdateDto {
            first_name: Some("UpdatedFirst".to_string()),
            last_name: Some("UpdatedLast".to_string()),
            birthday: Some(NaiveDate::from_ymd_opt(1985, 12, 25).unwrap()),
            address: Some("Updated Address 456".to_string()),
            gender: Some(Gender::Other),
        };

        // Act: Send PATCH request to update profile
        let request = Request::builder()
            .method(Method::PATCH)
            .uri("/profile")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(serde_json::to_string(&update_dto).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Verify the update was successful
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let updated_user: serde_json::Value = serde_json::from_str(&response_text).unwrap();
        let data = &updated_user["data"];

        println!("{}", serde_json::to_string_pretty(&updated_user).unwrap());
        assert_eq!(data["first_name"], "UpdatedFirst");
        assert_eq!(data["last_name"], "UpdatedLast");
        assert_eq!(data["birthday"], "1985-12-25");
        assert_eq!(data["address"], "Updated Address 456");
        assert_eq!(data["gender"], Gender::Other.to_string());

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_success_partial() {
        // Arrange: Create user and get token
        let (user, password) = create_test_user("e2e_update_profile_success_partial").await;
        let token = login_and_get_token(user.email.clone(), password.clone()).await;
        let app = create_user_router().await;

        let update_dto = ProfileUpdateDto {
            first_name: Some("PartialFirst".to_string()),
            last_name: None,
            birthday: None,
            address: Some("Partial Address Update".to_string()),
            gender: None,
        };

        // Act: Send PATCH request with partial update
        let request = Request::builder()
            .method(Method::PATCH)
            .uri("/profile")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(serde_json::to_string(&update_dto).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Verify only specified fields were updated
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let updated_user: serde_json::Value = serde_json::from_str(&response_text).unwrap();
        let data = &updated_user["data"];

        assert_eq!(data["first_name"], "PartialFirst");
        assert_eq!(data["address"], "Partial Address Update");
        // Other fields should remain unchanged from the original user

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_validation_error_empty_address() {
        // Arrange: Create user and get token
        let (user, password) = create_test_user("e2e_update_profile_validation").await;
        let token = login_and_get_token(user.email.clone(), password.clone()).await;
        let app = create_user_router().await;

        let invalid_update_dto = json!({
            "first_name": null,
            "last_name": null,
            "birthday": null,
            "address": "", // Invalid empty address
            "gender": null
        });

        // Act: Send PATCH request with invalid data
        let request = Request::builder()
            .method(Method::PATCH)
            .uri("/profile")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(invalid_update_dto.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Verify validation error
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        assert!(response_json.get("message").is_some(), "Error response should contain message");

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_validation_error_address_too_long() {
        // Arrange: Create user and get token
        let (user, password) = create_test_user("e2e_update_profile_long_addr").await;
        let token = login_and_get_token(user.email.clone(), password.clone()).await;
        let app = create_user_router().await;

        let long_address = "a".repeat(257); // Over 256 characters
        let invalid_update_dto = json!({
            "first_name": null,
            "last_name": null,
            "birthday": null,
            "address": long_address,
            "gender": null
        });

        // Act: Send PATCH request with invalid data
        let request = Request::builder()
            .method(Method::PATCH)
            .uri("/profile")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(invalid_update_dto.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Verify validation error
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        assert!(response_json.get("message").is_some(), "Error response should contain message");

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_no_updates() {
        // Arrange: Create user and get token
        let (user, password) = create_test_user("e2e_update_profile_no_fields").await;
        let token = login_and_get_token(user.email.clone(), password.clone()).await;
        let app = create_user_router().await;

        let empty_update_dto = UserFactory::fake_user_update_dto_empty();

        // Act: Send PATCH request with no updates
        let request = Request::builder()
            .method(Method::PATCH)
            .uri("/profile")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(serde_json::to_string(&empty_update_dto).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Verify no updates error
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        assert!(response_json.get("message").is_some(), "Error response should contain message");

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_unauthorized_no_token() {
        // Arrange: Create app but don't log in
        let app = create_user_router().await;

        let update_dto = ProfileUpdateDto {
            first_name: Some("UnauthorizedFirst".to_string()),
            last_name: None,
            birthday: None,
            address: None,
            gender: None,
        };

        // Act: Send PATCH request without authorization header
        let request = Request::builder()
            .method(Method::PATCH)
            .uri("/profile")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&update_dto).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Verify unauthorized error
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_unauthorized_invalid_token() {
        // Arrange: Create app
        let app = create_user_router().await;

        let update_dto = ProfileUpdateDto {
            first_name: Some("InvalidTokenFirst".to_string()),
            last_name: None,
            birthday: None,
            address: None,
            gender: None,
        };

        // Act: Send PATCH request with invalid token
        let request = Request::builder()
            .method(Method::PATCH)
            .uri("/profile")
            .header("content-type", "application/json")
            .header("authorization", "Bearer invalid_token_here")
            .body(Body::from(serde_json::to_string(&update_dto).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Verify unauthorized error
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_with_all_genders() {
        // Arrange: Create user and get token
        let (user, password) = create_test_user("e2e_update_profile_with_all_genders").await;
        let token = login_and_get_token(user.email.clone(), password.clone()).await;
        let app = create_user_router().await;

        // Test updating to each gender
        let genders = all_genders();
        for (i, gender) in genders.iter().enumerate() {
            let update_dto = ProfileUpdateDto {
                first_name: Some(format!("User{}", i)),
                last_name: None,
                birthday: None,
                address: None,
                gender: Some(gender.clone()),
            };

            // Act: Update the user
            let request = Request::builder()
                .method(Method::PATCH)
                .uri("/profile")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(serde_json::to_string(&update_dto).unwrap()))
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();

            // Assert: Verify the update was successful
            assert_eq!(response.status(), StatusCode::OK, "Gender update should succeed for {:?}", gender);

            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let response_text = String::from_utf8(body.to_vec()).unwrap();
            let updated_user: serde_json::Value = serde_json::from_str(&response_text).unwrap();
            let data = &updated_user["data"];

            assert_eq!(data["gender"], genders[i].clone().to_string());
            assert_eq!(data["first_name"], format!("User{}", i));
        }

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_malformed_json() {
        // Arrange: Create user and get token
        let (user, password) = create_test_user("e2e_update_profile_malformed_json").await;
        let token = login_and_get_token(user.email.clone(), password.clone()).await;
        let app = create_user_router().await;

        // Act: Send PATCH request with malformed JSON
        let request = Request::builder()
            .method(Method::PATCH)
            .uri("/profile")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from("{ invalid json here }"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Verify bad request error
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_wrong_content_type() {
        // Arrange: Create user and get token
        let (user, password) = create_test_user("e2e_update_profile_wrong_content_type").await;
        let token = login_and_get_token(user.email.clone(), password.clone()).await;
        let app = create_user_router().await;

        let update_dto = ProfileUpdateDto {
            first_name: Some("WrongContentType".to_string()),
            last_name: None,
            birthday: None,
            address: None,
            gender: None,
        };

        // Act: Send PATCH request with wrong content type
        let request = Request::builder()
            .method(Method::PATCH)
            .uri("/profile")
            .header("content-type", "text/plain")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(serde_json::to_string(&update_dto).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Verify unsupported media type or bad request
        assert!(
            response.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE ||
            response.status() == StatusCode::BAD_REQUEST
        );

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_update_profile_preserves_unchanged_fields() {
        // Arrange: Create user and get token
        let (user, password) = create_test_user("e2e_update_profile_preserve").await;
        let token = login_and_get_token(user.email.clone(), password.clone()).await;
        let app = create_user_router().await;

        // Get current profile to compare
        let get_request = Request::builder()
            .method(Method::GET)
            .uri("/profile")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let get_response = app.clone().oneshot(get_request).await.unwrap();
        assert_eq!(get_response.status(), StatusCode::OK);

        let body = to_bytes(get_response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let original_user: serde_json::Value = serde_json::from_str(&response_text).unwrap();
        let data1 = &original_user["data"];

        let original_username = data1["username"].as_str().unwrap();
        let original_email = data1["email"].as_str().unwrap();
        let original_last_name = data1["last_name"].as_str().unwrap();

        // Update only first_name
        let update_dto = ProfileUpdateDto {
            first_name: Some("PreserveTest".to_string()),
            last_name: None,
            birthday: None,
            address: None,
            gender: None,
        };

        // Act: Send PATCH request to update only first_name
        let patch_request = Request::builder()
            .method(Method::PATCH)
            .uri("/profile")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(serde_json::to_string(&update_dto).unwrap()))
            .unwrap();

        let patch_response = app.oneshot(patch_request).await.unwrap();

        // Assert: Verify unchanged fields are preserved
        assert_eq!(patch_response.status(), StatusCode::OK);

        let body = to_bytes(patch_response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let updated_user: serde_json::Value = serde_json::from_str(&response_text).unwrap();
        let data2 = &updated_user["data"];

        assert_eq!(data2["first_name"], "PreserveTest");
        assert_eq!(data2["username"], original_username);
        assert_eq!(data2["email"], original_email);
        assert_eq!(data2["last_name"], original_last_name);

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }
}
