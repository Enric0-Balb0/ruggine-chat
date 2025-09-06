use sqlx::Error;
use chrono::{DateTime, Utc};
use crate::config::database::DatabaseTrait;
use crate::entity::cpu_usage_log::CpuUsageLog;
use crate::repository::cpu_usage_log_repository::cpu_usage_log_repository::CpuUsageLogRepository;

impl CpuUsageLogRepository {
    pub async fn find_paginated_inner(&self, cursor: Option<DateTime<Utc>>, limit: usize) -> Result<Vec<CpuUsageLog>, Error> {
        let messages = match cursor {
            Some(cursor_datetime) => {
                sqlx::query_as::<_, CpuUsageLog>(
                    r#"
                    SELECT id, timestamp, cpu_usage_percent
                    FROM cpu_usage_log
                    WHERE timestamp < $1
                    ORDER BY timestamp DESC
                    LIMIT $2
                    "#
                )
                .bind(cursor_datetime)
                .bind(limit as i64)
                .fetch_all(self.db_conn.get_pool())
                .await?
            }
            None => {
                // First page - get latest messages
                sqlx::query_as::<_, CpuUsageLog>(
                    r#"
                    SELECT id, timestamp, cpu_usage_percent
                    FROM cpu_usage_log
                    ORDER BY timestamp DESC
                    LIMIT $1
                    "#
                )
                .bind(limit as i64)
                .fetch_all(self.db_conn.get_pool())
                .await?
            }
        };

        Ok(messages)
    }
}
