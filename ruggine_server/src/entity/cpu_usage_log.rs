use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dto::cpu_usage_log_dto::CpuUsageLogCreateDto;
#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow, Default, PartialEq)]
pub struct CpuUsageLog {
    pub id: i32,
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: BigDecimal,
}

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow, Default, PartialEq)]
pub struct NewCpuUsageLog {
    pub cpu_usage_percent: BigDecimal,
}

impl From<CpuUsageLogCreateDto> for NewCpuUsageLog {
    fn from(dto: CpuUsageLogCreateDto) -> Self {
        NewCpuUsageLog {
            cpu_usage_percent: dto.cpu_usage_percent,
        }
    }
}