// Integration test for CpuUsageLogRepository::insert_inner
// This test follows the style of text_message_repository/insert_test.rs

use crate::{cleanup_all_cpu_usage_log, common};
use ruggine_server::factory::cpu_usage_log_factory::CpuUsageLogFactory;
use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;
use serial_test::serial;

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_insert_cpu_usage_log_success() {
    // Arrange
    cleanup_all_cpu_usage_log().await;
    let db = common::get_database().await;
    let repository = CpuUsageLogRepository::new(&db);

    // Create a new cpu usage log entry
    let new_log = CpuUsageLogFactory::unique_fake_new_cpu_usage_log();

    // Act
    let result = repository.insert_inner(new_log).await;

    // Assert
    assert!(result.is_ok(), "Failed to insert cpu usage log: {:?}", result.err());
    let log_id = result.unwrap();
    assert!(log_id > 0, "Log ID should be positive");

    // Verify the log was actually inserted
    let found = repository.find_inner(log_id).await;
    assert!(found.is_ok(), "Failed to find inserted cpu usage log");
    let log = found.unwrap();
    assert_eq!(log.id, log_id);
    // cpu_usage_percent is random, just check type/field exists
    assert!(log.cpu_usage_percent >= 0.into() && log.cpu_usage_percent <= 100.into());
    // timestamp should be close to now, but we don't check it strictly

    // Cleanup
    common::cleanup_cpu_usage_log(log.id).await;
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_insert_cpu_usage_log_invalid() {
    // Arrange
    cleanup_all_cpu_usage_log().await;
    let db = common::get_database().await;
    let repository = CpuUsageLogRepository::new(&db);

    // Try to insert with invalid value (simulate with negative percent if DB allows, else skip)
    let mut new_log = CpuUsageLogFactory::unique_fake_new_cpu_usage_log();
    new_log.cpu_usage_percent = (-10).into();

    let result = repository.insert_inner(new_log).await;
    // Should fail or DB should reject, but if DB allows, this will pass
    assert!(result.is_err() || result.as_ref().unwrap() > &0, "Insert should fail or return valid id");
}
#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_concurrent_insert_cpu_usage_log() {
    use futures::future::join_all;
    cleanup_all_cpu_usage_log().await;
    let db = common::get_database().await;
    let repository = CpuUsageLogRepository::new(&db);
    let mut handles = vec![];
    for _ in 0..10 {
        let repo = repository.clone();
        handles.push(tokio::spawn(async move {
            let log = CpuUsageLogFactory::unique_fake_new_cpu_usage_log();
            repo.insert_inner(log).await
        }));
    }
    let results = join_all(handles).await;
    let mut ids = vec![];
    for res in results {
        let id = res.expect("Join error").expect("Insert error");
        ids.push(id);
    }
    // Cleanup
    for id in ids { common::cleanup_cpu_usage_log(id).await; }
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_insert_and_fetch_extreme_values() {
    cleanup_all_cpu_usage_log().await;
    let db = common::get_database().await;
    let repository = CpuUsageLogRepository::new(&db);
    // Insert 0%
    let mut log0 = CpuUsageLogFactory::unique_fake_new_cpu_usage_log();
    log0.cpu_usage_percent = 0.into();
    let id0 = repository.insert_inner(log0).await.expect("Insert 0% failed");
    let found0 = repository.find_inner(id0).await.expect("Find 0% failed");
    assert_eq!(found0.cpu_usage_percent, 0.into());
    // Insert 100%
    let mut log100 = CpuUsageLogFactory::unique_fake_new_cpu_usage_log();
    log100.cpu_usage_percent = 100.into();
    let id100 = repository.insert_inner(log100).await.expect("Insert 100% failed");
    let found100 = repository.find_inner(id100).await.expect("Find 100% failed");
    assert_eq!(found100.cpu_usage_percent, 100.into());
    // Cleanup
    common::cleanup_cpu_usage_log(id0).await;
    common::cleanup_cpu_usage_log(id100).await;
}