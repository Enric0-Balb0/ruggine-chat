use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Deserialize, Serialize, Validate, ToSchema, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CpuUsageLogPaginationQuery {
    /// Cursor for pagination (timestamp)
    #[param(example = "2025-08-27T12:34:56Z")] // esempio per Swagger
    #[schema(value_type = String)]
    pub cursor: Option<DateTime<Utc>>,

    /// Number of items to return (max 100)
    #[validate(range(min = 1, max = 100))]
    #[serde(default = "default_limit")]
    #[param(example = 20, minimum = 1, maximum = 100, default = 20)]
    pub limit: usize,
}
impl CpuUsageLogPaginationQuery {
    pub fn new(cursor: Option<DateTime<Utc>>, limit: usize) -> Self {
        Self { cursor, limit }
    }
}

fn default_limit() -> usize {
    20
}

/// DTO for finding cpu usage logs by group with pagination
#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, Validate)]
pub struct FindCpuUsageLogsByGroupQuery {
    /// Group chat ID to filter by
    #[validate(range(min = 1))]
    pub group_chat_id: i32,

    /// Pagination parameters
    #[serde(flatten)]
    pub pagination: CpuUsageLogPaginationQuery,
}

impl FindCpuUsageLogsByGroupQuery {
    pub fn new(group_chat_id: i32, cursor: Option<DateTime<Utc>>, limit: usize) -> Self {
        Self {
            group_chat_id,
            pagination: CpuUsageLogPaginationQuery::new(cursor, limit),
        }
    }
}
