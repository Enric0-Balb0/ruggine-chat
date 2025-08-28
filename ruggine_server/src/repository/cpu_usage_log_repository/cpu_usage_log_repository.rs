use crate::config::database::{Database, DatabaseTrait};
use crate::entity::cpu_usage_log::{CpuUsageLog, NewCpuUsageLog};
pub use crate::repository::cpu_usage_log_repository::cpu_usage_log_repository_trait::CpuUsageLogRepositoryTrait;
use async_trait::async_trait;
use sqlx::Error;
use std::sync::Arc;

#[derive(Clone)]
pub struct CpuUsageLogRepository {
    pub(crate) db_conn: Arc<Database>,
}

impl CpuUsageLogRepository {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    pub async fn delete_by_id(&self, id: i32) -> Result<i32, Error> {
        let rec = sqlx::query_scalar(
            r#"
            DELETE FROM cpu_usage_log
            WHERE id = $1
            RETURNING id
            "#
        )
        .bind(id)
        .fetch_one(self.db_conn.get_pool())
        .await?;

        Ok(rec)
    }

    pub async fn delete_all(&self) -> Result<Vec<i32>, Error> {
        let rec = sqlx::query_scalar(
            r#"
        DELETE FROM cpu_usage_log
        RETURNING id
        "#
        )
            .fetch_all(self.db_conn.get_pool())
            .await?;

        Ok(rec)
    }
}

#[async_trait]
impl CpuUsageLogRepositoryTrait for CpuUsageLogRepository {
    async fn find(&self, id: i32) -> Result<CpuUsageLog, Error> {
        self.find_inner(id).await
    }

    async fn insert(&self, new_text_message: NewCpuUsageLog) -> Result<i32, Error> {
        self.insert_inner(new_text_message).await
    }

    async fn find_paginated(&self, cursor: Option<chrono::DateTime<chrono::Utc>>, limit: usize) -> Result<Vec<CpuUsageLog>, Error> {
        self.find_paginated_inner(cursor, limit).await
    }
    fn db_conn(&self) -> Arc<Database> {
        self.db_conn.clone()
    }
}
