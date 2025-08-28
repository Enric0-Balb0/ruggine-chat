use std::sync::Arc;
use async_trait::async_trait;
use crate::dto::cpu_usage_log_dto::{CpuUsageLogCreateDto, CpuUsageLogReadDto};
use crate::dto::cpu_usage_log_pagination_dto::CpuUsageLogPaginationQuery;
use crate::error::api_error::ApiError;
use crate::response::PaginatedCpuUsageLogResponse;
pub use crate::service::cpu_usage_log_service::cpu_usage_log_service::CpuUsageLogService;
pub use crate::service::cpu_usage_log_service::cpu_usage_log_service_trait::CpuUsageLogServiceTrait;

mod cpu_usage_log_service_trait;
mod cpu_usage_log_service;
mod find_by_id;
mod find_paginated;
mod create;
mod cpu_monitor_taks;

#[async_trait]
impl CpuUsageLogServiceTrait for CpuUsageLogService {
    async fn find_by_id(&self, id: i32) -> Result<CpuUsageLogReadDto, ApiError> {
        self.find_by_id_internal(id).await
    }

    async fn find_paginated(
        &self,
        pagination_query: CpuUsageLogPaginationQuery,
    ) -> Result<PaginatedCpuUsageLogResponse, ApiError> {
        self.find_paginated_internal(pagination_query).await
    }

    async fn create(
        &self,
        payload: CpuUsageLogCreateDto,
    ) -> Result<CpuUsageLogReadDto, ApiError> {
        self.create_internal(payload).await
    }

    async fn start_monitoring(&self) -> Result<(), ApiError> {
        let self_arc = Arc::new(self.clone_for_arc());
        self_arc.start_monitoring_internal().await
    }

    async fn stop_monitoring(&mut self) -> Result<(), ApiError> {
        self.stop_monitoring_internal().await
    }

    fn set_monitoring_interval_ms(&mut self, interval_ms: u64) {
        self.set_monitoring_interval_ms_internal(interval_ms);
    }
}