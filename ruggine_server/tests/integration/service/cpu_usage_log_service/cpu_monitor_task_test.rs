use std::sync::Arc;
use std::time::Duration;
use bigdecimal::BigDecimal;
use serial_test::serial;
use tokio::time::timeout;
use tracing::log::log;
use ruggine_server::dto::cpu_usage_log_pagination_dto::CpuUsageLogPaginationQuery;
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::cpu_usage_log_error::CpuUsageLogError;
use ruggine_server::factory::cpu_usage_log_factory::CpuUsageLogFactory;
use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;
use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository_trait::CpuUsageLogRepositoryTrait;
use ruggine_server::service::cpu_usage_log_service::{CpuUsageLogService, CpuUsageLogServiceTrait};
use crate::common;

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_start_monitoring_success() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let mut service = CpuUsageLogService::new(cpu_usage_log_repo);
    
    // Set a monitoring interval before starting
    service.set_monitoring_interval_ms(10); // Short interval for testing
    
    // Check initial state
    assert!(!service.is_running().await, "Should not be running initially");
    
    // Act - start monitoring
    let result = service.start_monitoring().await;
    
    // Assert
    assert!(result.is_ok(), "Failed to start monitoring: {:?}", result.err());

    tokio::time::sleep(Duration::from_millis(1000)).await;

    assert!(service.stop_monitoring().await.is_ok());

    let logs = service.find_paginated(CpuUsageLogPaginationQuery {
        cursor: None,
        limit: 1000
    }).await.unwrap();

    assert!(logs.data.len() >= 30);

    for log in logs.data {
        common::cleanup_cpu_usage_log(log.id).await;
    }
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_start_monitoring_without_interval_fails() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);
    
    // Note: Not setting monitoring interval
    
    // Check initial state
    assert!(!service.is_running().await, "Should not be running initially");
    
    // Act - start monitoring without setting interval
    let result = service.start_monitoring().await;
    
    // Assert
    assert!(result.is_err(), "Should fail when interval is not set");
    match result.unwrap_err() {
        ApiError::CpuUsageLogError(CpuUsageLogError::CpuMonitoringIntervalMustBeSet) => {
            // Expected error
        },
        other => panic!("Expected CpuMonitoringIntervalMustBeSet, got: {:?}", other),
    }
    
    // Verify still not running
    assert!(!service.is_running().await, "Should still not be running after failure");
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_start_monitoring_twice_fails() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let mut service = CpuUsageLogService::new(cpu_usage_log_repo);
    
    // Set monitoring interval
    service.set_monitoring_interval_ms(100);
    
    // Start monitoring first time
    let result1 = service.start_monitoring().await;
    assert!(result1.is_ok(), "First start should succeed");
    assert!(service.is_running().await, "Should be running after first start");
    
    // Act - try to start monitoring again
    let result2 = service.start_monitoring().await;
    
    // Assert
    assert!(result2.is_err(), "Second start should fail");
    match result2.unwrap_err() {
        ApiError::CpuUsageLogError(CpuUsageLogError::CpuMonitoringAlreadyRunning) => {
            // Expected error
        },
        other => panic!("Expected CpuMonitoringAlreadyRunning, got: {:?}", other),
    }
    
    // Cleanup - stop monitoring
    assert!(service.stop_monitoring().await.is_ok(), "Should be able to stop monitoring");
    assert!(!service.is_running().await, "Should not be running after stop");
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_stop_monitoring_when_not_running_fails() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let mut service = CpuUsageLogService::new(cpu_usage_log_repo);
    
    // Check initial state
    assert!(!service.is_running().await, "Should not be running initially");
    
    // Act - try to stop monitoring when not running
    let result = service.stop_monitoring().await;
    
    // Assert
    assert!(result.is_err(), "Should fail when trying to stop non-running monitoring");
    match result.unwrap_err() {
        ApiError::CpuUsageLogError(CpuUsageLogError::CpuMonitoringNotRunning) => {
            // Expected error
        },
        other => panic!("Expected CpuMonitoringNotRunning, got: {:?}", other),
    }
    
    // Verify still not running
    assert!(!service.is_running().await, "Should still not be running after failed stop");
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_monitoring_state_consistency() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let mut service = CpuUsageLogService::new(cpu_usage_log_repo);
    
    // Set a short monitoring interval
    service.set_monitoring_interval_ms(50);
    
    // Initial state
    assert!(!service.is_running().await, "Should not be running initially");
    
    // Start monitoring
    let start_result = service.start_monitoring().await;
    assert!(start_result.is_ok(), "Should start successfully");
    assert!(service.is_running().await, "Should be running after start");
    
    // Wait a short time to ensure monitoring is active
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Verify still running
    assert!(service.is_running().await, "Should still be running after some time");
    
    // Stop monitoring
    let stop_result = service.stop_monitoring().await;
    assert!(stop_result.is_ok(), "Should stop successfully");
    assert!(!service.is_running().await, "Should not be running after stop");
    
    // Try to stop again (should fail)
    let stop_again_result = service.stop_monitoring().await;
    assert!(stop_again_result.is_err(), "Second stop should fail");
    
    // Cleanup any logs that might have been created
    let logs = service.find_paginated(CpuUsageLogPaginationQuery {
        cursor: None,
        limit: 100
    }).await.unwrap();
    
    for log in logs.data {
        common::cleanup_cpu_usage_log(log.id).await;
    }
}
