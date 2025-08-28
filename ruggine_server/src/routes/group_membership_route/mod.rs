pub mod find_by_id_and_user_id;
pub mod find_by_user_id;
pub mod leave_group;
pub mod find_by_group_chat_id;
pub mod find_connected_users_and_online;

use axum::Router;
use crate::entity::user::all_user_types;

pub fn routes(group_membership_state: crate::state::group_membership_state::GroupMembershipState, token_state: crate::state::token_state::TokenState) -> Router {
    use tower::ServiceBuilder;
    use axum::middleware;

    Router::new()
        .merge(
            find_by_id_and_user_id::routes()
                .with_state(group_membership_state.clone())
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(token_state.clone(), crate::middleware::auth_middleware::auth(all_user_types())),
                ))
        )
        .merge(
            find_by_user_id::routes()
                .with_state(group_membership_state.clone())
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(token_state.clone(), crate::middleware::auth_middleware::auth(all_user_types())),
                ))
        )
        .merge(
            leave_group::routes()
                .with_state(group_membership_state.clone())
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(token_state.clone(), crate::middleware::auth_middleware::auth(all_user_types())),
                ))
        )
        .merge(
            find_by_group_chat_id::routes()
                .with_state(group_membership_state.clone())
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(token_state.clone(), crate::middleware::auth_middleware::auth(all_user_types())),
                ))
        )
        .merge(
            find_connected_users_and_online::routes()
                .with_state(group_membership_state)
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(token_state, crate::middleware::auth_middleware::auth(all_user_types())),
                ))
        )
}
