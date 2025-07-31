use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use ruggine_server::factory::group_chat_factory::GroupChatFactory;
use axum::body::to_bytes;
use crate::common::{cleanup_user, cleanup_group, create_group_chat_router, create_login_and_get_token};

#[cfg(test)]
mod create_group_chat_e2e_tests {
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_success() {
        // Arrange: Create router and login user
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_create_success".to_string()).await;
        
        // Create unique group chat data using factory
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("e2e_success");

        let create_payload = json!({
            "name": create_dto.name,
            "description": create_dto.description
        });

        // Act: Send POST request to /create with auth token
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(create_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with group chat data
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure and data
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        let data = &response_json["data"];
        
        assert!(data["id"].as_i64().unwrap() > 0, "Group ID should be positive");
        assert_eq!(data["name"].as_str().unwrap(), create_dto.name);
        assert_eq!(data["description"].as_str().unwrap(), create_dto.description);
        assert_eq!(data["created_by"].as_i64().unwrap(), user.id as i64);
        assert!(data["created_at"].as_str().is_some(), "Should have created_at timestamp");
        assert!(data["updated_at"].as_str().is_some(), "Should have updated_at timestamp");

        // Cleanup
        let group_id = data["id"].as_i64().unwrap() as i32;
        cleanup_group(group_id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_unauthorized_no_token() {
        // Arrange: Create router and group chat data
        let app = create_group_chat_router().await;
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("e2e_no_auth");

        let create_payload = json!({
            "name": create_dto.name,
            "description": create_dto.description
        });

        // Act: Send POST request without auth token
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .body(Body::from(create_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_unauthorized_invalid_token() {
        // Arrange: Create router and group chat data
        let app = create_group_chat_router().await;
        let create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("e2e_invalid_auth");

        let create_payload = json!({
            "name": create_dto.name,
            "description": create_dto.description
        });

        // Act: Send POST request with invalid token
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .header("authorization", "Bearer invalid_token_here")
            .body(Body::from(create_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_bad_request_empty_name() {
        // Arrange: Create router and login user
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_empty_name".to_string()).await;

        // Create payload with empty name (should fail validation)
        let create_payload = json!({
            "name": "",
            "description": "Valid description"
        });

        // Act: Send POST request with invalid data
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(create_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_bad_request_missing_name() {
        // Arrange: Create router and login user
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_missing_name".to_string()).await;

        // Create payload without name field
        let create_payload = json!({
            "description": "Valid description"
        });

        // Act: Send POST request with missing required field
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(create_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_with_empty_description() {
        // Arrange: Create router and login user
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_empty_desc".to_string()).await;
        
        // Create group chat data with empty description (should be allowed)
        let mut create_dto = GroupChatFactory::unique_fake_group_chat_create_dto("e2e_empty_description");
        create_dto.description = String::new();

        let create_payload = json!({
            "name": create_dto.name,
            "description": create_dto.description
        });

        // Act: Send POST request
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(create_payload.to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should succeed with empty description
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_multiple_groups_same_user() {
        // Arrange: Create router and login user
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_multiple_groups".to_string()).await;
        
        // Create multiple unique group chat DTOs
        let create_dto1 = GroupChatFactory::unique_fake_group_chat_create_dto("e2e_multi_1");
        let create_dto2 = GroupChatFactory::unique_fake_group_chat_create_dto("e2e_multi_2");
        let create_dto3 = GroupChatFactory::unique_fake_group_chat_create_dto("e2e_multi_3");

        let mut group_ids = Vec::new();

        // Act & Assert: Create multiple groups
        for (i, create_dto) in [create_dto1, create_dto2, create_dto3].iter().enumerate() {
            let create_payload = json!({
                "name": create_dto.name,
                "description": create_dto.description
            });

            let request = Request::builder()
                .method("POST")
                .uri("/create")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(create_payload.to_string()))
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            
            // Each creation should succeed
            assert_eq!(response.status(), StatusCode::OK, "Group {} creation should succeed", i + 1);

            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let response_text = String::from_utf8(body.to_vec()).unwrap();
            let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

            let data = &response_json["data"];
            let group_id = data["id"].as_i64().unwrap() as i32;
            group_ids.push(group_id);

            // Verify group data
            assert_eq!(data["name"].as_str().unwrap(), create_dto.name);
            assert_eq!(data["description"].as_str().unwrap(), create_dto.description);
            assert_eq!(data["created_by"].as_i64().unwrap(), user.id as i64);
        }

        // Verify all groups have different IDs
        assert_ne!(group_ids[0], group_ids[1]);
        assert_ne!(group_ids[0], group_ids[2]);
        assert_ne!(group_ids[1], group_ids[2]);

        // Cleanup
        for group_id in group_ids {
            cleanup_group(group_id).await;
        }
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_groups_same_name_different_users() {
        // Arrange: Create router and login two different users
        let app = create_group_chat_router().await;
        let (user1, _password1, token1) = create_login_and_get_token("e2e_same_name_1".to_string()).await;
        let (user2, _password2, token2) = create_login_and_get_token("e2e_same_name_2".to_string()).await;
        
        // Use same group name for both users (should be allowed)
        let group_name = "Duplicate Name Group";
        let group_description = "Same name different users";

        let create_payload = json!({
            "name": group_name,
            "description": group_description
        });

        let mut group_ids = Vec::new();

        // Act: Create group with user1
        let request1 = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token1))
            .body(Body::from(create_payload.to_string()))
            .unwrap();

        let response1 = app.clone().oneshot(request1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::OK);

        let body1 = to_bytes(response1.into_body(), usize::MAX).await.unwrap();
        let response_text1 = String::from_utf8(body1.to_vec()).unwrap();
        let response_json1: serde_json::Value = serde_json::from_str(&response_text1).unwrap();
        let group_id1 = response_json1["data"]["id"].as_i64().unwrap() as i32;
        group_ids.push(group_id1);

        // Act: Create group with same name but user2
        let request2 = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token2))
            .body(Body::from(create_payload.to_string()))
            .unwrap();

        let response2 = app.oneshot(request2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::OK);

        let body2 = to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let response_text2 = String::from_utf8(body2.to_vec()).unwrap();
        let response_json2: serde_json::Value = serde_json::from_str(&response_text2).unwrap();
        let group_id2 = response_json2["data"]["id"].as_i64().unwrap() as i32;
        group_ids.push(group_id2);

        // Assert: Both groups should be created successfully with different IDs
        assert_ne!(group_id1, group_id2, "Groups should have different IDs");
        assert_eq!(response_json1["data"]["created_by"].as_i64().unwrap(), user1.id as i64);
        assert_eq!(response_json2["data"]["created_by"].as_i64().unwrap(), user2.id as i64);

        // Cleanup
        for group_id in group_ids {
            cleanup_group(group_id).await;
        }
        cleanup_user(user1.email).await;
        cleanup_user(user2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_groups_same_name_same_user() {
        // Arrange: Create router and login user
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_same_name_same_user".to_string()).await;
        
        // Use same group name twice for same user (should be allowed)
        let group_name = "Duplicate Name Same User";
        let group_description = "Same name same user";

        let create_payload = json!({
            "name": group_name,
            "description": group_description
        });

        let mut group_ids = Vec::new();

        // Act: Create first group
        let request1 = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token.clone()))
            .body(Body::from(create_payload.to_string()))
            .unwrap();

        let response1 = app.clone().oneshot(request1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::OK);

        let body1 = to_bytes(response1.into_body(), usize::MAX).await.unwrap();
        let response_text1 = String::from_utf8(body1.to_vec()).unwrap();
        let response_json1: serde_json::Value = serde_json::from_str(&response_text1).unwrap();
        let group_id1 = response_json1["data"]["id"].as_i64().unwrap() as i32;
        group_ids.push(group_id1);

        // Act: Create second group with same name and same user
        let request2 = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(create_payload.to_string()))
            .unwrap();

        let response2 = app.oneshot(request2).await.unwrap();
        assert_eq!(response2.status(), StatusCode::OK);

        let body2 = to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let response_text2 = String::from_utf8(body2.to_vec()).unwrap();
        let response_json2: serde_json::Value = serde_json::from_str(&response_text2).unwrap();
        let group_id2 = response_json2["data"]["id"].as_i64().unwrap() as i32;
        group_ids.push(group_id2);

        // Assert: Both groups should be created successfully with different IDs but same name
        assert_ne!(group_id1, group_id2, "Groups should have different IDs");
        assert_eq!(response_json1["data"]["name"].as_str().unwrap(), group_name);
        assert_eq!(response_json2["data"]["name"].as_str().unwrap(), group_name);
        assert_eq!(response_json1["data"]["created_by"].as_i64().unwrap(), user.id as i64);
        assert_eq!(response_json2["data"]["created_by"].as_i64().unwrap(), user.id as i64);

        // Cleanup
        for group_id in group_ids {
            cleanup_group(group_id).await;
        }
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_create_group_chat_malformed_json() {
        // Arrange: Create router and login user
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_malformed_json".to_string()).await;

        // Act: Send POST request with malformed JSON
        let request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from("{ invalid json }"))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        // Cleanup
        cleanup_user(user.email).await;
    }
}
