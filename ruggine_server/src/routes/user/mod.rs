pub mod profile;
pub mod register;

use axum::Router;

pub fn routes(user_state: crate::state::user_state::UserState, token_state: crate::state::token_state::TokenState) -> Router {
    use tower::ServiceBuilder;
    use axum::middleware;

    Router::new()
        .merge(
            register::routes()
                .with_state(user_state.clone()) // state per il register
        )
        .merge(
            profile::routes()
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(token_state, crate::middleware::auth_middleware::auth),
                ))
        )
}
