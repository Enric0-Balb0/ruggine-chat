use axum::Router;
use crate::entity::user::UserType;

pub mod find_paginated;

pub fn routes(
    cpu_usage_log_state: crate::state::cpu_usage_log_state::CpuUsageLogState,
    token_state: crate::state::token_state::TokenState,
) -> Router {
    use tower::ServiceBuilder;
    use axum::middleware;

    Router::new()
        .merge(
            find_paginated::routes()
                .with_state(cpu_usage_log_state.clone())
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(
                        token_state.clone(),
                        crate::middleware::auth_middleware::auth(vec![UserType::Admin, UserType::Developer]),
                    ),
                ))
        )
}
