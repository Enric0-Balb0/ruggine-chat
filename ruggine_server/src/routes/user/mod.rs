pub mod profile_route;
pub mod register_route;

use axum::Router;

use crate::entity::user::all_user_types;

pub fn routes(user_state: crate::state::user_state::UserState, token_state: crate::state::token_state::TokenState) -> Router {
    use tower::ServiceBuilder;
    use axum::middleware;

    Router::new()
        .merge(
            register_route::routes()
                .with_state(user_state.clone()) // state per il register
        )
        .merge(
            profile_route::routes()
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(token_state, crate::middleware::auth_middleware::auth(all_user_types())),
                ))
        )
}
