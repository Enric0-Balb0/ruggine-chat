use ruggine_server::dto::cpu_usage_log_pagination_dto::CpuUsageLogPaginationQuery;
use ruggine_server::factory::cpu_usage_log_factory::CpuUsageLogFactory;
use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;
use ruggine_server::service::cpu_usage_log_service::{CpuUsageLogService, CpuUsageLogServiceTrait};
use bigdecimal::BigDecimal;
use chrono::{Duration, Utc};
use crate::common;
use std::sync::Arc;
use serial_test::serial;

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_paginated_success_without_cursor() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    // Create test data with unique CPU percentages for this test (10-14 range)
    let mut created_log_ids = Vec::new();
    for i in 0..5 {
        let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(10 + i));
        let created_log = service.create(payload).await.unwrap();
        created_log_ids.push(created_log.id);
        
        // Small delay to ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    let pagination_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_limit(3);

    // Act
    let result = service.find_paginated(pagination_query).await;

    // Assert
    assert!(result.is_ok(), "Failed to find paginated CPU usage logs: {:?}", result.err());
    let response = result.unwrap();
    
    assert!(response.data.len() >= 3); // Should return at least our 3 items (might have more from other tests)
    assert_eq!(response.pagination.page_size, response.data.len());

    // Verify logs are in descending timestamp order (most recent first)
    for i in 1..response.data.len() {
        assert!(response.data[i-1].timestamp >= response.data[i].timestamp);
    }

    // Cleanup
    for id in created_log_ids {
        common::cleanup_cpu_usage_log(id).await;
    }
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_paginated_with_cursor() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    // Create test data with unique CPU percentages for this test (20-24 range)
    let mut created_log_ids = Vec::new();
    
    for i in 0..5 {
        let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(20 + i));
        let created_log = service.create(payload).await.unwrap();
        created_log_ids.push(created_log.id);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // First page without cursor
    let first_page_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_limit(2);
    let first_page = service.find_paginated(first_page_query).await.unwrap();

    // Use cursor from first page for second page
    let cursor = first_page.pagination.next_cursor;

    if let Some(cursor_value) = cursor {
        let second_page_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_cursor(cursor_value);
        
        // Act
        let result = service.find_paginated(second_page_query).await;

        // Assert
        assert!(result.is_ok());
        let second_page = result.unwrap();
        
        // Second page should not contain items from first page
        for first_item in &first_page.data {
            for second_item in &second_page.data {
                assert_ne!(first_item.id, second_item.id);
            }
        }
        
        // All items in second page should have timestamps older than cursor
        for item in &second_page.data {
            assert!(item.timestamp < cursor_value);
        }
    }

    // Cleanup
    for id in created_log_ids {
        common::cleanup_cpu_usage_log(id).await;
    }
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_paginated_empty_result() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    // Use a cursor far in the past where no logs should exist
    let old_cursor = Utc::now() - Duration::days(365);
    let pagination_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_cursor(old_cursor);

    // Act
    let result = service.find_paginated(pagination_query).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    
    assert_eq!(response.data.len(), 0);
    assert!(!response.pagination.has_more);
    assert!(response.pagination.next_cursor.is_none());
    assert_eq!(response.pagination.page_size, 0);
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_paginated_limit_boundary() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    // Create exactly 3 logs with unique CPU percentages for this test (30-32 range)
    let mut created_log_ids = Vec::new();
    for i in 0..3 {
        let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(30 + i));
        let created_log = service.create(payload).await.unwrap();
        created_log_ids.push(created_log.id);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Request exactly the same number as created
    let pagination_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_limit(3);

    // Act
    let result = service.find_paginated(pagination_query).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    
    assert!(response.data.len() >= 3); // Should return at least our 3 items
    assert_eq!(response.pagination.page_size, response.data.len());

    // Cleanup
    for id in created_log_ids {
        common::cleanup_cpu_usage_log(id).await;
    }
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_paginated_large_limit() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    // Create a few logs with unique CPU percentages for this test (40-42 range)
    let mut created_log_ids = Vec::new();
    for i in 0..3 {
        let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(40 + i));
        let created_log = service.create(payload).await.unwrap();
        created_log_ids.push(created_log.id);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Request much more than available
    let pagination_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_limit(100);

    // Act
    let result = service.find_paginated(pagination_query).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    
    // Should return all available logs (not more than created)
    assert!(response.data.len() >= 3); // At least the 3 we created (might have more from other tests)
    assert_eq!(response.pagination.page_size, response.data.len());

    // Cleanup
    for id in created_log_ids {
        common::cleanup_cpu_usage_log(id).await;
    }
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_paginated_chronological_order() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    // Create logs with known CPU usage percentages in order (50-54 range for this test)
    let mut created_log_ids = Vec::new();
    let cpu_percentages = vec![50, 51, 52, 53, 54];
    
    for percentage in cpu_percentages.clone() {
        let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(percentage));
        let created_log = service.create(payload).await.unwrap();
        created_log_ids.push(created_log.id);
        
        // Ensure different timestamps
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    let pagination_query = CpuUsageLogFactory::fake_cpu_usage_log_pagination_query_with_limit(10);

    // Act
    let result = service.find_paginated(pagination_query).await;

    // Assert
    assert!(result.is_ok());
    let response = result.unwrap();
    
    // Find our created logs in the response (filter by CPU usage percentages we know)
    let our_logs: Vec<_> = response.data.iter()
        .filter(|log| {
            let cpu_val = log.cpu_usage_percent.to_string().parse::<i32>().unwrap_or(-1);
            cpu_percentages.contains(&cpu_val)
        })
        .collect();

    // Verify they are in descending timestamp order (most recent first, which means highest percentage first)
    for i in 1..our_logs.len() {
        assert!(our_logs[i-1].timestamp >= our_logs[i].timestamp);
    }

    // Cleanup
    for id in created_log_ids {
        common::cleanup_cpu_usage_log(id).await;
    }
}
