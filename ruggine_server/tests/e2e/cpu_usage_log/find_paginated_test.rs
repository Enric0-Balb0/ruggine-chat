use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use axum::body::to_bytes;
use serial_test::serial;
use serde_json::json;
use chrono::{Utc, Duration};
use bigdecimal::BigDecimal;
use ruggine_server::factory::cpu_usage_log_factory::CpuUsageLogFactory;
use crate::common::{
    cleanup_user_by_email, create_full_router,
    create_admin_login_and_get_token, create_developer_login_and_get_token,
    create_login_and_get_token, cleanup_cpu_usage_log, create_test_cpu_usage_log,
    cleanup_all_cpu_usage_log,
};

#[cfg(test)]
mod find_paginated_cpu_usage_log_e2e_tests {
    use super::*;

    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_find_paginated_success_as_admin() {
        // Arrange: Create router, admin user and CPU usage logs
        cleanup_all_cpu_usage_log().await;
        let app = create_full_router().await;
        let (user, _password, token) = create_admin_login_and_get_token("e2e_cpu_admin".to_string()).await;
        
        // Create test CPU usage logs with different percentages
        let log1 = create_test_cpu_usage_log(25.5).await;
        let log2 = create_test_cpu_usage_log(45.2).await;
        let log3 = create_test_cpu_usage_log(78.9).await;

        // Act: Send GET request to / with auth token and pagination params
        let request = Request::builder()
            .method("GET")
            .uri("/api/cpu_usage_log?limit=10")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with paginated CPU usage logs
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify response structure
        assert!(response_json.get("data").is_some(), "Response should contain data field");
        assert!(response_json.get("pagination").is_some(), "Response should contain pagination field");
        
        let data = response_json["data"].as_array().unwrap();
        assert!(data.len() >= 3, "Should return at least the 3 test logs we created");
        
        // Verify pagination structure
        let pagination = &response_json["pagination"];
        assert!(pagination.get("has_more").is_some());
        assert!(pagination.get("next_cursor").is_some() || pagination["next_cursor"].is_null());
        assert!(pagination.get("page_size").is_some());
        
        // Verify that our test logs are included (check by cpu_percent values)
        let mut found_test_logs = 0;
        for log_data in data {
            assert!(log_data["id"].as_i64().is_some());
            assert!(log_data["cpu_usage_percent"].as_str().is_some());
            assert!(log_data["timestamp"].as_str().is_some());
            
            if let Some(cpu_percent_str) = log_data["cpu_usage_percent"].as_str() {
                let cpu_percent: f64 = cpu_percent_str.parse().unwrap_or(0.0);
                if (cpu_percent - 25.5).abs() < 0.1 || 
                   (cpu_percent - 45.2).abs() < 0.1 || 
                   (cpu_percent - 78.9).abs() < 0.1 {
                    found_test_logs += 1;
                }
            }
        }
        
        // We should find at least our test logs, but there might be others from concurrent tests
        assert!(found_test_logs >= 1, "Should find at least one of our test CPU logs");

        // Cleanup
        cleanup_cpu_usage_log(log1.id).await;
        cleanup_cpu_usage_log(log2.id).await;
        cleanup_cpu_usage_log(log3.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_find_paginated_success_as_developer() {
        // Arrange: Create router, developer user and CPU usage logs
        cleanup_all_cpu_usage_log().await;
        let app = create_full_router().await;
        let (user, _password, token) = create_developer_login_and_get_token("e2e_cpu_dev".to_string()).await;
        
        // Create a test CPU usage log
        let log = create_test_cpu_usage_log(33.7).await;

        // Act: Send GET request with developer token
        let request = Request::builder()
            .method("GET")
            .uri("/api/cpu_usage_log?limit=5")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        // Verify basic response structure
        assert!(response_json.get("data").is_some());
        assert!(response_json.get("pagination").is_some());
        
        let pagination = &response_json["pagination"];
        assert_eq!(pagination["page_size"].as_i64().unwrap(), 1);

        // Cleanup
        cleanup_cpu_usage_log(log.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_find_paginated_unauthorized_regular_user() {
        // Arrange: Create router and regular user (not admin/developer)
        cleanup_all_cpu_usage_log().await;
        let app = create_full_router().await;
        let (user, _password, token) = create_login_and_get_token("e2e_cpu_regular".to_string()).await;

        // Act: Send GET request with regular user token
        let request = Request::builder()
            .method("GET")
            .uri("/api/cpu_usage_log?limit=10")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 403 Forbidden (or 401 Unauthorized)
        assert!(
            response.status() == StatusCode::FORBIDDEN || 
            response.status() == StatusCode::UNAUTHORIZED,
            "Regular users should not have access to CPU usage logs"
        );

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_find_paginated_unauthorized_no_token() {
        // Arrange: Create router only
        cleanup_all_cpu_usage_log().await;
        let app = create_full_router().await;

        // Act: Send GET request without auth token
        let request = Request::builder()
            .method("GET")
            .uri("/api/cpu_usage_log?limit=10")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        
        // Should return proper error message
        assert!(response_text.contains("Unauthorized") || response_text.contains("Missing") || response_text.contains("Invalid"));
    }

    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_find_paginated_with_pagination_params() {
        // Arrange: Create router, admin user and multiple CPU usage logs
        cleanup_all_cpu_usage_log().await;
        let app = create_full_router().await;
        let (user, _password, token) = create_admin_login_and_get_token("e2e_cpu_paginate".to_string()).await;
        
        // Create multiple test logs to test pagination
        let mut logs = Vec::new();
        for i in 1..=7 {
            let log = create_test_cpu_usage_log((i as f32) * 10.0).await;
            logs.push(log);
            // Add small delay to ensure different timestamps
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Act: Send GET request with small limit to test pagination
        let request = Request::builder()
            .method("GET")
            .uri("/api/cpu_usage_log?limit=3")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK with limited results
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json["data"].as_array().unwrap();
        let pagination = &response_json["pagination"];
        
        // Should respect the limit
        assert_eq!(pagination["page_size"].as_i64().unwrap(), 3);
        
        // Data should be returned in reverse chronological order (newest first)
        if data.len() >= 2 {
            let first_timestamp = data[0]["timestamp"].as_str().unwrap();
            let second_timestamp = data[1]["timestamp"].as_str().unwrap();
            
            let first_time = chrono::DateTime::parse_from_rfc3339(first_timestamp).unwrap();
            let second_time = chrono::DateTime::parse_from_rfc3339(second_timestamp).unwrap();
            
            assert!(first_time >= second_time, "Results should be in reverse chronological order");
        }

        // Cleanup
        for log in logs {
            cleanup_cpu_usage_log(log.id).await;
        }
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_find_paginated_with_cursor() {
        // Arrange: Create router, admin user and CPU usage logs
        cleanup_all_cpu_usage_log().await;
        let app = create_full_router().await;
        let (user, _password, token) = create_admin_login_and_get_token("e2e_cpu_cursor".to_string()).await;
        
        // Create test logs with specific timing
        let log1 = create_test_cpu_usage_log(11.1).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let log2 = create_test_cpu_usage_log(22.2).await;

        // First request to get the cursor
        let request1 = Request::builder()
            .method("GET")
            .uri("/api/cpu_usage_log?limit=1")
            .header("authorization", format!("Bearer {}", token.clone()))
            .body(Body::empty())
            .unwrap();

        let response1 = app.oneshot(request1).await.unwrap();
        assert_eq!(response1.status(), StatusCode::OK);

        let body1 = to_bytes(response1.into_body(), usize::MAX).await.unwrap();
        let response1_text = String::from_utf8(body1.to_vec()).unwrap();
        let response1_json: serde_json::Value = serde_json::from_str(&response1_text).unwrap();

        let pagination1 = &response1_json["pagination"];
        let next_cursor = pagination1["next_cursor"].as_str();

        // Act: Use cursor for second request if available
        if let Some(cursor) = next_cursor {
            let app2 = create_full_router().await; // Need new app instance
            
            let request2 = Request::builder()
                .method("GET")
                .uri(&format!("/api/cpu_usage_log?limit=1&cursor={}", cursor))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap();

            let response2 = app2.oneshot(request2).await.unwrap();

            // Assert: Should return 200 OK with next page
            assert_eq!(response2.status(), StatusCode::OK);

            let body2 = to_bytes(response2.into_body(), usize::MAX).await.unwrap();
            let response2_text = String::from_utf8(body2.to_vec()).unwrap();
            let response2_json: serde_json::Value = serde_json::from_str(&response2_text).unwrap();

            let data2 = response2_json["data"].as_array().unwrap();
            
            // Should have data (might be our logs or others from concurrent tests)
            assert!(data2.len() <= 1, "Should respect limit parameter");
        }

        // Cleanup
        cleanup_cpu_usage_log(log1.id).await;
        cleanup_cpu_usage_log(log2.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_find_paginated_invalid_limit_too_high() {
        // Arrange: Create router and admin user
        cleanup_all_cpu_usage_log().await;
        let app = create_full_router().await;
        let (user, _password, token) = create_admin_login_and_get_token("e2e_cpu_limit_high".to_string()).await;

        // Act: Send GET request with limit higher than max (100)
        let request = Request::builder()
            .method("GET")
            .uri("/api/cpu_usage_log?limit=150")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request due to validation error
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        
        // Should contain validation error message
        assert!(response_text.contains("limit") || response_text.contains("100") || response_text.contains("validation"));

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_find_paginated_invalid_limit_too_low() {
        // Arrange: Create router and admin user
        cleanup_all_cpu_usage_log().await;
        let app = create_full_router().await;
        let (user, _password, token) = create_admin_login_and_get_token("e2e_cpu_limit_low".to_string()).await;

        // Act: Send GET request with limit of 0 (below min of 1)
        let request = Request::builder()
            .method("GET")
            .uri("/api/cpu_usage_log?limit=0")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request due to validation error
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        
        // Should contain validation error message
        assert!(response_text.contains("limit") || response_text.contains("1") || response_text.contains("validation"));

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_find_paginated_invalid_cursor_format() {
        // Arrange: Create router and admin user
        cleanup_all_cpu_usage_log().await;
        let app = create_full_router().await;
        let (user, _password, token) = create_admin_login_and_get_token("e2e_cpu_cursor_invalid".to_string()).await;

        // Act: Send GET request with invalid cursor format
        let request = Request::builder()
            .method("GET")
            .uri("/api/cpu_usage_log?limit=10&cursor=invalid_timestamp_format")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 400 Bad Request due to invalid cursor
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_find_paginated_without_limit() {
        // Arrange: Create router and admin user
        cleanup_all_cpu_usage_log().await;
        let app = create_full_router().await;
        let (user, _password, token) = create_admin_login_and_get_token("e2e_cpu_default".to_string()).await;

        // Create test logs with specific timing
        let log1 = create_test_cpu_usage_log(11.1).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let log2 = create_test_cpu_usage_log(22.2).await;

        // Act: Send GET request without limit parameter (should use default)
        let request = Request::builder()
            .method("GET")
            .uri("/api/cpu_usage_log")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json["data"].as_array().unwrap();
        let pagination = &response_json["pagination"];

        // Should respect the limit
        assert_eq!(pagination["page_size"].as_i64().unwrap(), 2);
        assert_eq!(data.len(), 2, "There should be 2 log elements");

        // Cleanup
        cleanup_cpu_usage_log(log1.id).await;
        cleanup_cpu_usage_log(log2.id).await;
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_find_paginated_no_logs() {
        // Arrange: Create router and admin user
        cleanup_all_cpu_usage_log().await;
        let app = create_full_router().await;
        let (user, _password, token) = create_admin_login_and_get_token("e2e_cpu_default".to_string()).await;

        // Act: Send GET request without limit parameter (should use default)
        let request = Request::builder()
            .method("GET")
            .uri("/api/cpu_usage_log")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 200 OK
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response_text = String::from_utf8(body.to_vec()).unwrap();
        let response_json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

        let data = response_json["data"].as_array().unwrap();
        let pagination = &response_json["pagination"];

        // Should respect the limit
        assert_eq!(pagination["page_size"].as_i64().unwrap(), 0);
        assert_eq!(data.len(), 0, "There should be 0 log elements");

        // Cleanup
        cleanup_user_by_email(user.email).await;
    }

    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_find_paginated_invalid_token() {
        // Arrange: Create router only
        cleanup_all_cpu_usage_log().await;
        let app = create_full_router().await;

        // Act: Send GET request with invalid token format
        let request = Request::builder()
            .method("GET")
            .uri("/api/cpu_usage_log?limit=10")
            .header("authorization", "Bearer invalid_token_format")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio_shared_rt::test(shared)]
    #[serial]
    async fn test_find_paginated_malformed_auth_header() {
        // Arrange: Create router only
        cleanup_all_cpu_usage_log().await;
        let app = create_full_router().await;

        // Act: Send GET request with malformed authorization header
        let request = Request::builder()
            .method("GET")
            .uri("/api/cpu_usage_log?limit=10")
            .header("authorization", "InvalidFormat token")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // Assert: Should return 401 Unauthorized
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
