use ruggine_server::factory::cpu_usage_log_factory::CpuUsageLogFactory;
use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;
use ruggine_server::service::cpu_usage_log_service::{CpuUsageLogService, CpuUsageLogServiceTrait};
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::cpu_usage_log_error::CpuUsageLogError;
use bigdecimal::{BigDecimal, FromPrimitive};
use crate::{cleanup_all_cpu_usage_log, common};
use std::sync::Arc;
use serial_test::serial;

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_by_id_success() {
    // Arrange
    cleanup_all_cpu_usage_log().await;
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    // First create a CPU usage log to find
    let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(65));
    let created_log = service.create(payload.clone()).await.unwrap();

    // Act
    let result = service.find_by_id(created_log.id).await;

    // Assert
    assert!(result.is_ok(), "Failed to find CPU usage log by ID: {:?}", result.err());
    let found_log = result.unwrap();
    
    assert_eq!(found_log.id, created_log.id);
    assert_eq!(found_log.cpu_usage_percent, payload.cpu_usage_percent);
    assert_eq!(found_log.timestamp, created_log.timestamp);

    // Cleanup
    common::cleanup_cpu_usage_log(created_log.id).await;
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_by_id_not_found() {
    // Arrange
    cleanup_all_cpu_usage_log().await;
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    let nonexistent_id = 99999;

    // Act
    let result = service.find_by_id(nonexistent_id).await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, ApiError::CpuUsageLogError(CpuUsageLogError::CpuUsageLogNotFound)));
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_by_id_multiple_logs() {
    // Arrange
    cleanup_all_cpu_usage_log().await;
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    // Create multiple CPU usage logs with unique percentages for this test (60-62 range)
    let payload1 = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(60));
    let payload2 = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(61));
    let payload3 = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(62));

    let created_log1 = service.create(payload1.clone()).await.unwrap();
    let created_log2 = service.create(payload2.clone()).await.unwrap();
    let created_log3 = service.create(payload3.clone()).await.unwrap();

    // Act - Find each log by its ID
    let result1 = service.find_by_id(created_log1.id).await;
    let result2 = service.find_by_id(created_log2.id).await;
    let result3 = service.find_by_id(created_log3.id).await;

    // Assert
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());

    let found_log1 = result1.unwrap();
    let found_log2 = result2.unwrap();
    let found_log3 = result3.unwrap();

    // Verify each log has correct data
    assert_eq!(found_log1.id, created_log1.id);
    assert_eq!(found_log1.cpu_usage_percent, BigDecimal::from(60));

    assert_eq!(found_log2.id, created_log2.id);
    assert_eq!(found_log2.cpu_usage_percent, BigDecimal::from(61));

    assert_eq!(found_log3.id, created_log3.id);
    assert_eq!(found_log3.cpu_usage_percent, BigDecimal::from(62));

    // Cleanup
    common::cleanup_cpu_usage_log(created_log1.id).await;
    common::cleanup_cpu_usage_log(created_log2.id).await;
    common::cleanup_cpu_usage_log(created_log3.id).await;
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_by_id_with_zero_percent() {
    // Arrange
    cleanup_all_cpu_usage_log().await;
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    // Create a CPU usage log with 0% usage
    let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(0));
    let created_log = service.create(payload.clone()).await.unwrap();

    // Act
    let result = service.find_by_id(created_log.id).await;

    // Assert
    assert!(result.is_ok());
    let found_log = result.unwrap();
    
    assert_eq!(found_log.id, created_log.id);
    assert_eq!(found_log.cpu_usage_percent, BigDecimal::from(0));

    // Cleanup
    common::cleanup_cpu_usage_log(created_log.id).await;
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_by_id_with_decimal_percent() {
    // Arrange
    cleanup_all_cpu_usage_log().await;
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    // Create a CPU usage log with decimal percentage
    let decimal_value = BigDecimal::from_f64(78.45).unwrap();
    let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(decimal_value.clone());
    let created_log = service.create(payload.clone()).await.unwrap();

    // Act
    let result = service.find_by_id(created_log.id).await;

    // Assert
    assert!(result.is_ok());
    let found_log = result.unwrap();
    
    assert_eq!(found_log.id, created_log.id);
    assert_eq!(found_log.cpu_usage_percent, decimal_value);

    // Cleanup
    common::cleanup_cpu_usage_log(created_log.id).await;
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_by_id_after_create_and_find_consistency() {
    // Arrange
    cleanup_all_cpu_usage_log().await;
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(55));

    // Act
    let created_log = service.create(payload.clone()).await.unwrap();
    let found_log = service.find_by_id(created_log.id).await.unwrap();

    // Assert - The log found should be identical to the created log
    assert_eq!(created_log.id, found_log.id);
    assert_eq!(created_log.cpu_usage_percent, found_log.cpu_usage_percent);
    assert_eq!(created_log.timestamp, found_log.timestamp);

    // Cleanup
    common::cleanup_cpu_usage_log(created_log.id).await;
}
