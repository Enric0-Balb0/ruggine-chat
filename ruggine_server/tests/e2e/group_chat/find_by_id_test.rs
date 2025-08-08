use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use axum::body::to_bytes;
use crate::common::{cleanup_user, cleanup_group_chat, create_group_chat_router, create_login_and_get_token, create_test_group_chat};

#[cfg(test)]
mod find_by_id_group_chat_e2e_tests {
    use crate::clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id;
    use super::*;

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_group_chat_success() {
        // Arrange: Create router, login user and create group chat
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_find_success".to_string()).await;
        let group = create_test_group_chat("e2e_find_success", user.id).await;

        // Act: Send GET request to /{id} with auth token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/{}", group.id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
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
        
        assert_eq!(data["id"].as_i64().unwrap(), group.id as i64);
        assert_eq!(data["name"].as_str().unwrap(), group.name);
        assert_eq!(data["description"].as_str().unwrap(), group.description);
        assert_eq!(data["created_by"].as_i64().unwrap(), group.created_by as i64);
        assert!(data["created_at"].as_str().is_some(), "Should have created_at timestamp");
        assert!(data["updated_at"].as_str().is_some(), "Should have updated_at timestamp");

        // Cleanup
        cleanup_group_chat(group.id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_group_chat_unauthorized_no_token() {
        // Arrange: Create router, user and group chat
        let app = create_group_chat_router().await;
        let (user, _password, _token) = create_login_and_get_token("e2e_find_no_auth".to_string()).await;
        let group = create_test_group_chat("e2e_find_no_auth", user.id).await;

        // Act: Send GET request without auth token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/{}", group.id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        cleanup_group_chat(group.id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_group_chat_unauthorized_invalid_token() {
        // Arrange: Create router, user and group chat
        let app = create_group_chat_router().await;
        let (user, _password, _token) = create_login_and_get_token("e2e_find_invalid_auth".to_string()).await;
        let group = create_test_group_chat("e2e_find_invalid_auth", user.id).await;

        // Act: Send GET request with invalid token
        let request = Request::builder()
            .method("GET")
            .uri(format!("/{}", group.id))
            .header("authorization", "Bearer invalid_token_here")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        // Cleanup
        cleanup_group_chat(group.id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_group_chat_not_found() {
        // Arrange: Create router and login user (but no group chat)
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_find_not_found".to_string()).await;
        let non_existent_id = 99999;

        // Act: Send GET request for non-existent group
        let request = Request::builder()
            .method("GET")
            .uri(format!("/{}", non_existent_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 404 Not Found or 500 Internal Server Error
        assert!(
            response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
            "Expected 404 or 500, got: {}", response.status()
        );

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_group_chat_invalid_id() {
        // Arrange: Create router and login user
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_find_invalid_id".to_string()).await;
        let invalid_id = -1;

        // Act: Send GET request with invalid ID
        let request = Request::builder()
            .method("GET")
            .uri(format!("/{}", invalid_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return error status (404)
        assert_eq!(response.status(), StatusCode::NOT_FOUND, "Expected404, for invalid ID, got: {}", response.status());

        // Cleanup
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_multiple_groups_same_user() {
        // Arrange: Create router, login user and create multiple groups
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_find_multiple".to_string()).await;
        let group1 = create_test_group_chat("e2e_find_multiple_1", user.id).await;
        let group2 = create_test_group_chat("e2e_find_multiple_2", user.id).await;
        let group3 = create_test_group_chat("e2e_find_multiple_3", user.id).await;

        // Act & Assert: Find each group
        for (i, group) in [&group1, &group2, &group3].iter().enumerate() {
            let request = Request::builder()
                .method("GET")
                .uri(format!("/{}", group.id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            
            // Each find should succeed
            assert_eq!(response.status(), StatusCode::OK, "Group {} find should succeed", i + 1);

            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let response_text = String::from_utf8(body.to_vec()).unwrap();
            let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

            let data = &response_json["data"];
            assert_eq!(data["id"].as_i64().unwrap(), group.id as i64);
            assert_eq!(data["name"].as_str().unwrap(), group.name);
            assert_eq!(data["created_by"].as_i64().unwrap(), user.id as i64);
        }

        // Cleanup
        cleanup_group_chat(group1.id).await;
        cleanup_group_chat(group2.id).await;
        cleanup_group_chat(group3.id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_different_users_same_group() {
        // Arrange: Create router, login two users and create one group
        let app = create_group_chat_router().await;
        let (user1, _password1, token1) = create_login_and_get_token("e2e_find_diff_user1".to_string()).await;
        let (user2, _password2, token2) = create_login_and_get_token("e2e_find_diff_user2".to_string()).await;
        let group = create_test_group_chat("e2e_find_shared", user1.id).await;

        // Act: Both users try to find the same group
        let request1 = Request::builder()
            .method("GET")
            .uri(format!("/{}", group.id))
            .header("authorization", format!("Bearer {}", token1))
            .body(Body::empty())
            .unwrap();

        let request2 = Request::builder()
            .method("GET")
            .uri(format!("/{}", group.id))
            .header("authorization", format!("Bearer {}", token2))
            .body(Body::empty())
            .unwrap();

        let response1 = app.clone().oneshot(request1).await.unwrap();
        let response2 = app.oneshot(request2).await.unwrap();

        // Assert: Both should succeed
        assert_eq!(response1.status(), StatusCode::OK, "First user should find the group");
        assert_eq!(response2.status(), StatusCode::OK, "Second user should find the group");

        // Verify both responses contain the same group data
        let body1 = to_bytes(response1.into_body(), usize::MAX).await.unwrap();
        let response1_json: serde_json::Value = serde_json::from_str(&String::from_utf8(body1.to_vec()).unwrap()).unwrap();
        
        let body2 = to_bytes(response2.into_body(), usize::MAX).await.unwrap();
        let response2_json: serde_json::Value = serde_json::from_str(&String::from_utf8(body2.to_vec()).unwrap()).unwrap();

        let data1 = &response1_json["data"];
        let data2 = &response2_json["data"];
        
        assert_eq!(data1["id"], data2["id"]);
        assert_eq!(data1["name"], data2["name"]);
        assert_eq!(data1["created_by"].as_i64().unwrap(), user1.id as i64);

        // Cleanup
        cleanup_group_chat(group.id).await;
        cleanup_user(user1.email).await;
        cleanup_user(user2.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_after_creation() {
        // Arrange: Create router and login user
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_find_after_create".to_string()).await;
        
        // First create a group via API
        let create_payload = serde_json::json!({
            "name": "E2E Test Group After Create",
            "description": "Group created via API for find test"
        });

        let create_request = Request::builder()
            .method("POST")
            .uri("/create")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(create_payload.to_string()))
            .unwrap();

        let create_response = app.clone().oneshot(create_request).await.unwrap();
        assert_eq!(create_response.status(), StatusCode::OK, "Group creation should succeed");

        // Extract group ID from creation response
        let create_body = to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
        let create_response_json: serde_json::Value = serde_json::from_str(&String::from_utf8(create_body.to_vec()).unwrap()).unwrap();
        let group_id = create_response_json["data"]["id"].as_i64().unwrap() as i32;

        // Act: Find the created group
        let find_request = Request::builder()
            .method("GET")
            .uri(format!("/{}", group_id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let find_response = app.oneshot(find_request).await.unwrap();

        // Assert: Should find the group successfully
        assert_eq!(find_response.status(), StatusCode::OK, "Should find newly created group");

        let find_body = to_bytes(find_response.into_body(), usize::MAX).await.unwrap();
        let find_response_json: serde_json::Value = serde_json::from_str(&String::from_utf8(find_body.to_vec()).unwrap()).unwrap();

        let data = &find_response_json["data"];
        assert_eq!(data["id"].as_i64().unwrap(), group_id as i64);
        assert_eq!(data["name"].as_str().unwrap(), "E2E Test Group After Create");
        assert_eq!(data["description"].as_str().unwrap(), "Group created via API for find test");
        assert_eq!(data["created_by"].as_i64().unwrap(), user.id as i64);

        // Cleanup
        clean_up_group_invitation_with_membership_by_user_id_and_group_chat_id(user.id, group_id).await;
        cleanup_group_chat(group_id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_concurrent_requests() {
        // Arrange: Create router, login user and create group
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_find_concurrent".to_string()).await;
        let group = create_test_group_chat("e2e_find_concurrent", user.id).await;

        // Act: Make multiple concurrent find requests
        let mut handles = vec![];
        for _i in 0..5 {
            let app_clone = app.clone();
            let token_clone = token.clone();
            let group_id = group.id;
            
            let handle = tokio::spawn(async move {
                let request = Request::builder()
                    .method("GET")
                    .uri(format!("/{}", group_id))
                    .header("authorization", format!("Bearer {}", token_clone))
                    .body(Body::empty())
                    .unwrap();

                app_clone.oneshot(request).await
            });
            handles.push(handle);
        }

        // Wait for all requests to complete
        let results = futures::future::join_all(handles).await;

        // Assert: All requests should succeed
        for (i, result) in results.into_iter().enumerate() {
            // First level: Check if task panicked
            let response_result = result.expect(&format!("Request {} panicked", i));

            // Second level: Check Ok response
            let response = response_result.expect(&format!("Request {} failed", i));

            // Controllo codice HTTP
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "Request {} did not return 200 OK",
                i
            );

            let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let response_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

            let data = &response_json["data"];
            assert_eq!(data["id"].as_i64().unwrap(), group.id as i64);
            assert_eq!(data["name"].as_str().unwrap(), group.name);
        }

        // Cleanup
        cleanup_group_chat(group.id).await;
        cleanup_user(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    async fn test_find_by_id_zero_id() {
        // Arrange: Create router and login user
        let app = create_group_chat_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_find_zero_id".to_string()).await;

        // Act: Send GET request with ID 0
        let request = Request::builder()
            .method("GET")
            .uri("/0")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return error (not found or bad request)
        assert!(
            response.status() == StatusCode::NOT_FOUND 
            || response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::INTERNAL_SERVER_ERROR,
            "Expected error status for ID 0, got: {}", response.status()
        );

        // Cleanup
        cleanup_user(user.email).await;
    }
}
