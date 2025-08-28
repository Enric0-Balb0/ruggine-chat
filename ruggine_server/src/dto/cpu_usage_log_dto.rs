use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;
use chrono::{DateTime, Utc};
use crate::entity::cpu_usage_log::CpuUsageLog;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate, PartialEq)]
pub struct CpuUsageLogCreateDto {
    #[schema(example = 45.5)]
    pub cpu_usage_percent: BigDecimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CpuUsageLogReadDto {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "2025-08-12T10:30:00Z")]
    pub timestamp: DateTime<Utc>,
    #[schema(value_type = String)]
    #[schema(example = 12.5)]
    pub cpu_usage_percent: BigDecimal,
}

impl From<CpuUsageLog> for CpuUsageLogReadDto {
    fn from(log: CpuUsageLog) -> Self {
        CpuUsageLogReadDto {
            id: log.id,
            timestamp: log.timestamp,
            cpu_usage_percent: log.cpu_usage_percent,
        }
    }
}