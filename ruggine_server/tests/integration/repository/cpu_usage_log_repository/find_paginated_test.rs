// Integration test for CpuUsageLogRepository::find_paginated_inner
// This test follows the style of text_message_repository/find_by_group_chat_id_paginated_service_test.rs

use crate::common;
use ruggine_server::repository::cpu_usage_log_repository::cpu_usage_log_repository::{CpuUsageLogRepository, CpuUsageLogRepositoryTrait};
use ruggine_server::factory::cpu_usage_log_factory::CpuUsageLogFactory;
use chrono::{Utc, Duration};
use serial_test::serial;

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_cpu_usage_log_paginated() {
    // Arrange
    let db = common::get_database().await;
    let repository = CpuUsageLogRepository::new(&db);

    // Insert multiple logs
    let mut ids = Vec::new();
    for _ in 0..5 {
        let new_log = CpuUsageLogFactory::unique_fake_new_cpu_usage_log();
        let log_id = repository.insert_inner(new_log).await.expect("Insert failed");
        ids.push(log_id);
    }

    // Act: fetch the latest 3 logs (no cursor)
    let logs = repository.find_paginated_inner(None, 3).await.expect("Paginated fetch failed");

    // Assert
    assert_eq!(logs.len(), 3, "Should fetch 3 logs");
    assert!(logs[0].timestamp >= logs[1].timestamp && logs[1].timestamp >= logs[2].timestamp);

    // Act: fetch with cursor (timestamp of the second log)
    let cursor = logs[2].timestamp;
    let logs2 = repository.find_paginated_inner(Some(cursor), 2).await.expect("Paginated fetch with cursor failed");
    // Should fetch logs older than the cursor
    assert!(logs2.iter().all(|log| log.timestamp < cursor));

    // Cleanup
    for id in ids { common::cleanup_cpu_usage_log(id).await; }
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_cpu_usage_log_paginated_empty() {
    // Arrange
    let db = common::get_database().await;
    let repository = CpuUsageLogRepository::new(&db);

    // Act: fetch logs when table is empty or with very high cursor
    let logs = repository.find_paginated_inner(Some(chrono::Utc::now() + chrono::Duration::days(365)), 10).await.expect("Paginated fetch failed");

    // Assert
    assert!(logs.is_empty() || logs.iter().all(|log| log.timestamp < chrono::Utc::now()), "Should return empty or only past logs");
}

#[tokio_shared_rt::test(shared)]
#[serial]
async fn test_find_cpu_usage_log_paginated_limit_zero() {
    let db = crate::common::get_database().await;
    let repository = CpuUsageLogRepository::new(&db);
    let logs = repository.find_paginated_inner(None, 0).await.expect("Should not fail");
    assert!(logs.is_empty(), "Should return empty vec for limit 0");
}
