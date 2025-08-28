use std::sync::Arc;
use crate::dto::cpu_usage_log_dto::{CpuUsageLogCreateDto, CpuUsageLogReadDto};
use crate::dto::cpu_usage_log_pagination_dto::CpuUsageLogPaginationQuery;
use crate::error::api_error::ApiError;
use crate::response::PaginatedCpuUsageLogResponse;
use async_trait::async_trait;
use mockall::automock;

#[automock]
#[async_trait]
pub trait CpuUsageLogServiceTrait: Send + Sync {
    /// Find a cpu usage log by its ID
    async fn find_by_id(&self, id: i32) -> Result<CpuUsageLogReadDto, ApiError>;

    /// Find cpu usage logs  with pagination
    async fn find_paginated(
        &self,
        pagination_query: CpuUsageLogPaginationQuery,
    ) -> Result<PaginatedCpuUsageLogResponse, ApiError>;

    async fn create(
        &self, 
        payload: CpuUsageLogCreateDto,
    ) -> Result<CpuUsageLogReadDto, ApiError>;

    async fn start_monitoring(&self) -> Result<(), ApiError>;
    async fn stop_monitoring(&mut self) -> Result<(), ApiError>;
    fn set_monitoring_interval_ms(&mut self, interval_ms: u64);
}
