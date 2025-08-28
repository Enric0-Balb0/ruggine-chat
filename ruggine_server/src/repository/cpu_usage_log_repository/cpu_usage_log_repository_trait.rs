use crate::config::database::{Database, DatabaseTrait};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mockall::automock;
use sqlx::Error;
use sqlx::Error as SqlxError;
use std::sync::Arc;
use crate::entity::cpu_usage_log::{CpuUsageLog, NewCpuUsageLog};

#[async_trait]
#[automock]
pub trait CpuUsageLogRepositoryTrait: Send + Sync {
    async fn find(&self, id: i32) -> Result<CpuUsageLog, Error>;
    async fn insert(&self, new_text_message: NewCpuUsageLog) -> Result<i32, SqlxError>;
    async fn find_paginated(&self, cursor: Option<DateTime<Utc>>, limit: usize) -> Result<Vec<CpuUsageLog>, Error>;
    fn db_conn(&self) -> Arc<Database>;
}
