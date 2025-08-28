// Factory for NewCpuUsageLog for integration tests (for src/factory)
// This is for use in main code/tests, not just test-only

use std::sync::atomic::{AtomicU32, Ordering};
use crate::entity::cpu_usage_log::{CpuUsageLog, NewCpuUsageLog};
use crate::dto::cpu_usage_log_dto::{CpuUsageLogCreateDto, CpuUsageLogReadDto};
use crate::dto::cpu_usage_log_pagination_dto::CpuUsageLogPaginationQuery;
use bigdecimal::BigDecimal;
use rand::Rng;
use chrono::{DateTime, Utc};

// Counter to generate unique data in tests
static TEST_COUNTER: AtomicU32 = AtomicU32::new(1);

pub struct CpuUsageLogFactory;

impl CpuUsageLogFactory {
    // NewCpuUsageLog factory methods
    pub fn fake_new_cpu_usage_log() -> NewCpuUsageLog {
        let mut rng = rand::thread_rng();
        NewCpuUsageLog {
            cpu_usage_percent: BigDecimal::from(rng.gen_range(0..100)),
        }
    }

    pub fn unique_fake_new_cpu_usage_log() -> NewCpuUsageLog {
        let mut rng = rand::thread_rng();
        NewCpuUsageLog {
            cpu_usage_percent: BigDecimal::from(rng.gen_range(0..100)),
        }
    }

    pub fn fake_new_cpu_usage_log_with_percent(cpu_usage_percent: BigDecimal) -> NewCpuUsageLog {
        NewCpuUsageLog {
            cpu_usage_percent,
        }
    }

    // CpuUsageLog factory methods
    pub fn fake_cpu_usage_log() -> CpuUsageLog {
        let mut rng = rand::thread_rng();
        CpuUsageLog {
            id: 1,
            cpu_usage_percent: BigDecimal::from(rng.gen_range(0..100)),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn fake_cpu_usage_log_with_id(id: i32) -> CpuUsageLog {
        let mut rng = rand::thread_rng();
        CpuUsageLog {
            id,
            cpu_usage_percent: BigDecimal::from(rng.gen_range(0..100)),
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn fake_cpu_usage_log_with_percent(cpu_usage_percent: BigDecimal) -> CpuUsageLog {
        CpuUsageLog {
            id: 1,
            cpu_usage_percent,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn fake_cpu_usage_logs_paginated(count: usize) -> Vec<CpuUsageLog> {
        let mut logs = Vec::new();
        let base_time = Utc::now();
        let mut rng = rand::thread_rng();
        
        for i in 0..count {
            logs.push(CpuUsageLog {
                id: (i + 1) as i32,
                cpu_usage_percent: BigDecimal::from(rng.gen_range(0..100)),
                timestamp: base_time - chrono::Duration::minutes(i as i64),
            });
        }
        
        logs
    }

    pub fn unique_fake_cpu_usage_log() -> CpuUsageLog {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut rng = rand::thread_rng();
        CpuUsageLog {
            id: counter as i32,
            cpu_usage_percent: BigDecimal::from(rng.gen_range(0..100)),
            timestamp: chrono::Utc::now(),
        }
    }

    // DTO factory methods
    pub fn fake_cpu_usage_log_create_dto() -> CpuUsageLogCreateDto {
        let mut rng = rand::thread_rng();
        CpuUsageLogCreateDto {
            cpu_usage_percent: BigDecimal::from(rng.gen_range(0..100)),
        }
    }

    pub fn fake_cpu_usage_log_create_dto_with_percent(cpu_usage_percent: BigDecimal) -> CpuUsageLogCreateDto {
        CpuUsageLogCreateDto {
            cpu_usage_percent,
        }
    }

    pub fn fake_cpu_usage_log_create_dto_with_random_percent() -> CpuUsageLogCreateDto {
        let mut rng = rand::thread_rng();
        CpuUsageLogCreateDto {
            cpu_usage_percent: BigDecimal::from(rng.gen_range(0..100)),
        }
    }

    pub fn unique_fake_cpu_usage_log_create_dto() -> CpuUsageLogCreateDto {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut rng = rand::thread_rng();
        CpuUsageLogCreateDto {
            cpu_usage_percent: BigDecimal::from(rng.gen_range(0..100) + (counter % 100)),
        }
    }

    pub fn fake_cpu_usage_log_read_dto() -> CpuUsageLogReadDto {
        let mut rng = rand::thread_rng();
        CpuUsageLogReadDto {
            id: 1,
            timestamp: chrono::Utc::now(),
            cpu_usage_percent: BigDecimal::from(rng.gen_range(0..100)),
        }
    }

    pub fn fake_cpu_usage_log_read_dto_with_id(id: i32) -> CpuUsageLogReadDto {
        let mut rng = rand::thread_rng();
        CpuUsageLogReadDto {
            id,
            timestamp: chrono::Utc::now(),
            cpu_usage_percent: BigDecimal::from(rng.gen_range(0..100)),
        }
    }

    pub fn fake_cpu_usage_log_pagination_query() -> CpuUsageLogPaginationQuery {
        CpuUsageLogPaginationQuery::new(None, 20)
    }

    pub fn fake_cpu_usage_log_pagination_query_with_cursor(cursor: DateTime<Utc>) -> CpuUsageLogPaginationQuery {
        CpuUsageLogPaginationQuery::new(Some(cursor), 20)
    }

    pub fn fake_cpu_usage_log_pagination_query_with_limit(limit: usize) -> CpuUsageLogPaginationQuery {
        CpuUsageLogPaginationQuery::new(None, limit)
    }

    // Utility methods for customization
    pub fn with_cpu_usage_percent(mut new_log: NewCpuUsageLog, cpu_usage_percent: BigDecimal) -> NewCpuUsageLog {
        new_log.cpu_usage_percent = cpu_usage_percent;
        new_log
    }

    pub fn with_id(mut log: CpuUsageLog, id: i32) -> CpuUsageLog {
        log.id = id;
        log
    }

    pub fn with_timestamp(mut log: CpuUsageLog, timestamp: DateTime<Utc>) -> CpuUsageLog {
        log.timestamp = timestamp;
        log
    }

    pub fn with_cpu_usage_percent_log(mut log: CpuUsageLog, cpu_usage_percent: BigDecimal) -> CpuUsageLog {
        log.cpu_usage_percent = cpu_usage_percent;
        log
    }
}
