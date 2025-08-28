use ruggine_server::dto::cpu_usage_log_dto::CpuUsageLogCreateDto;
use ruggine_server::factory::cpu_usage_log_factory::CpuUsageLogFactory;
use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;
use ruggine_server::service::cpu_usage_log_service::{CpuUsageLogService, CpuUsageLogServiceTrait};
use ruggine_server::error::api_error::ApiError;
use ruggine_server::error::db_error::DbError;
use bigdecimal::{BigDecimal, FromPrimitive};
use crate::common;
use std::sync::Arc;
use serial_test::serial;

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_create_cpu_usage_log_success() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(75));

    // Act
    let result = service.create(payload.clone()).await;

    // Assert
    assert!(result.is_ok(), "Failed to create CPU usage log: {:?}", result.err());
    let created_log = result.unwrap();
    
    assert_eq!(created_log.cpu_usage_percent, payload.cpu_usage_percent);
    assert!(created_log.id > 0);
    assert!(created_log.timestamp <= chrono::Utc::now());

    // Cleanup
    common::cleanup_cpu_usage_log(created_log.id).await;
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_create_cpu_usage_log_with_zero_percent() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(0));

    // Act
    let result = service.create(payload.clone()).await;

    // Assert
    assert!(result.is_ok());
    let created_log = result.unwrap();
    
    assert_eq!(created_log.cpu_usage_percent, BigDecimal::from(0));
    assert!(created_log.id > 0);

    // Cleanup
    common::cleanup_cpu_usage_log(created_log.id).await;
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_create_cpu_usage_log_with_high_percent() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(100));

    // Act
    let result = service.create(payload.clone()).await;

    // Assert
    assert!(result.is_ok());
    let created_log = result.unwrap();
    
    assert_eq!(created_log.cpu_usage_percent, BigDecimal::from(100));
    assert!(created_log.id > 0);

    // Cleanup
    common::cleanup_cpu_usage_log(created_log.id).await;
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_create_cpu_usage_log_with_decimal_percent() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    let decimal_value = BigDecimal::from_f64(45.75).unwrap();
    let payload = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(decimal_value.clone());

    // Act
    let result = service.create(payload.clone()).await;

    // Assert
    assert!(result.is_ok());
    let created_log = result.unwrap();
    
    assert_eq!(created_log.cpu_usage_percent, decimal_value);
    assert!(created_log.id > 0);

    // Cleanup
    common::cleanup_cpu_usage_log(created_log.id).await;
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_create_multiple_cpu_usage_logs() {
    // Arrange
    let db = common::get_database().await;
    let cpu_usage_log_repo = Arc::new(CpuUsageLogRepository::new(&db));
    let service = CpuUsageLogService::new(cpu_usage_log_repo);

    let payload1 = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(25));
    let payload2 = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(50));
    let payload3 = CpuUsageLogFactory::fake_cpu_usage_log_create_dto_with_percent(BigDecimal::from(75));

    // Act
    let result1 = service.create(payload1.clone()).await;
    let result2 = service.create(payload2.clone()).await;
    let result3 = service.create(payload3.clone()).await;

    // Assert
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
    
    let log1 = result1.unwrap();
    let log2 = result2.unwrap();
    let log3 = result3.unwrap();

    // All logs should have different IDs
    assert_ne!(log1.id, log2.id);
    assert_ne!(log2.id, log3.id);
    assert_ne!(log1.id, log3.id);

    // All logs should have correct CPU usage percentages
    assert_eq!(log1.cpu_usage_percent, BigDecimal::from(25));
    assert_eq!(log2.cpu_usage_percent, BigDecimal::from(50));
    assert_eq!(log3.cpu_usage_percent, BigDecimal::from(75));

    // Log timestamps should be in chronological order (log1 <= log2 <= log3)
    assert!(log1.timestamp <= log2.timestamp);
    assert!(log2.timestamp <= log3.timestamp);

    // Cleanup
    common::cleanup_cpu_usage_log(log1.id).await;
    common::cleanup_cpu_usage_log(log2.id).await;
    common::cleanup_cpu_usage_log(log3.id).await;
}
