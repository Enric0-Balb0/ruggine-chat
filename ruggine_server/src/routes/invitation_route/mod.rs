pub mod send;
pub mod find_by_id_and_user_id;

use axum::Router;
use crate::entity::user::all_user_types;

pub fn routes(invitation_state: crate::state::invitation_state::InvitationState, token_state: crate::state::token_state::TokenState) -> Router {
    use tower::ServiceBuilder;
    use axum::middleware;

    Router::new()
        .merge(
            send::routes()
                .with_state(invitation_state.clone())
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(token_state.clone(), crate::middleware::auth_middleware::auth(all_user_types())),
                ))
        )
        .merge(
            find_by_id_and_user_id::routes()
                .with_state(invitation_state.clone())
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(token_state, crate::middleware::auth_middleware::auth(all_user_types())),
                ))
        )
}
