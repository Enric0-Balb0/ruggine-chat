// Integration test for CpuUsageLogRepository::find_inner
// This test follows the style of text_message_repository/find_by_id_test.rs

use serial_test::serial;
use crate::{cleanup_all_cpu_usage_log, common};
use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository::{CpuUsageLogRepository, CpuUsageLogRepositoryTrait};
use ruggine_server::factory::cpu_usage_log_factory::CpuUsageLogFactory;

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_cpu_usage_log_by_id() {
    // Arrange
    cleanup_all_cpu_usage_log().await;
    let db = common::get_database().await;
    let repository = CpuUsageLogRepository::new(&db);

    // Insert a new cpu usage log entry
    let new_log = CpuUsageLogFactory::unique_fake_new_cpu_usage_log();
    let log_id = repository.insert_inner(new_log.clone()).await.expect("Insert failed");

    // Act
    let found = repository.find_inner(log_id).await;

    // Assert
    assert!(found.is_ok(), "Failed to find cpu usage log by id: {:?}", found.err());
    let log = found.unwrap();
    assert_eq!(log.id, log_id);
    // cpu_usage_percent is random, just check type/field exists
    assert!(log.cpu_usage_percent >= 0.into() && log.cpu_usage_percent <= 100.into());

    // Cleanup
    common::cleanup_cpu_usage_log(log.id).await;
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_cpu_usage_log_by_id_not_found() {
    // Arrange
    cleanup_all_cpu_usage_log().await;
    let db = common::get_database().await;
    let repository = CpuUsageLogRepository::new(&db);

    // Act
    let found = repository.find_inner(-999999).await;

    // Assert
    assert!(found.is_err(), "Should not find log with invalid id");
}
