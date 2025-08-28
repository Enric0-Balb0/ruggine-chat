use axum::{
    routing::get,
    Router,
};

pub fn routes() -> Router<crate::state::cpu_usage_log_state::CpuUsageLogState> {
    Router::new()
        .route("/", get(crate::handler::cpu_usage_log_handler::find_paginated::find_paginated))
}
