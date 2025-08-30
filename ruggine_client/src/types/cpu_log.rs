use serde::{Deserialize, Serialize};

/// DTO for a single CPU usage log entry as returned by the API
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CpuUsageLogReadDto {
    pub id: i32,
    pub timestamp: String,
    pub cpu_usage_percent: String,
}

/// Pagination metadata returned by paginated endpoints
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub has_more: bool,
    pub next_cursor: Option<String>,
    pub page_size: i32,
    pub total_count: Option<i32>,
}

/// Paginated response wrapper for cpu usage logs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginatedCpuUsageLogResponse {
    pub data: Vec<CpuUsageLogReadDto>,
    pub pagination: Pagination,
}

/// Client-side model to surface a page of cpu usage logs
#[derive(Clone, Debug)]
pub struct CpuUsagePage {
    pub items: Vec<CpuUsageLogReadDto>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
    pub page_size: i32,
    pub total_count: Option<i32>,
}

impl From<PaginatedCpuUsageLogResponse> for CpuUsagePage {
    fn from(r: PaginatedCpuUsageLogResponse) -> Self {
        Self {
            items: r.data,
            has_more: r.pagination.has_more,
            next_cursor: r.pagination.next_cursor,
            page_size: r.pagination.page_size,
            total_count: r.pagination.total_count,
        }
    }
}
