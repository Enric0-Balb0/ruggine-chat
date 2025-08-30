use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CpuUsageLogError {
    #[error("Cpu usage log not found")]
    CpuUsageLogNotFound,
    #[error("CPU monitoring is already running")]
    CpuMonitoringAlreadyRunning,
    #[error("CPU monitoring not running")]
    CpuMonitoringNotRunning,
    #[error("CPU monitoring interval must be set")]
    CpuMonitoringIntervalMustBeSet,
}

impl IntoResponse for CpuUsageLogError {
    fn into_response(self) -> Response {
        let status_code = match self {
            CpuUsageLogError::CpuUsageLogNotFound => StatusCode::NOT_FOUND,
            CpuUsageLogError::CpuMonitoringAlreadyRunning => StatusCode::CONFLICT,
            CpuUsageLogError::CpuMonitoringNotRunning => StatusCode::BAD_REQUEST,
            CpuUsageLogError::CpuMonitoringIntervalMustBeSet => StatusCode::BAD_REQUEST,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
