use crate::dto::cpu_usage_log_pagination_dto::CpuUsageLogPaginationQuery;
use crate::entity::user::User;
use crate::error::api_error::ApiError;
use crate::error::request_error::ValidatedQuery;
use crate::response::PaginatedCpuUsageLogResponse;
use crate::state::cpu_usage_log_state::CpuUsageLogState;
use axum::{extract::State, Extension, Json};

#[utoipa::path(
    get,
    path = "/api/cpu_usage_log",
    params(
        CpuUsageLogPaginationQuery
    ),
    responses(
        (status = 200, description = "Cpu usage logs retrieved successfully", body = PaginatedCpuUsageLogResponse),
        (status = 401, description = "User cannot access logs"),
        (status = 500, description = "Internal server error")
    ),
    tag = "CpuUsageLog",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn find_paginated(
    Extension(_current_user): Extension<User>,
    State(state): State<CpuUsageLogState>,
    ValidatedQuery(pagination_query): ValidatedQuery<CpuUsageLogPaginationQuery>,
) -> Result<Json<PaginatedCpuUsageLogResponse>, ApiError> {
    let messages = state
        .cpu_usage_log_service
        .find_paginated(pagination_query)
        .await?;

    Ok(Json(messages))
}
