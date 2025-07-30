pub mod create;

use axum::Router;

use crate::entity::user::all_user_types;

pub fn routes(group_chat_state: crate::state::group_chat_state::GroupChatState, token_state: crate::state::token_state::TokenState) -> Router {
    use tower::ServiceBuilder;
    use axum::middleware;

    Router::new()
        .merge(
            create::routes()
                .with_state(group_chat_state.clone())
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(token_state, crate::middleware::auth_middleware::auth(all_user_types())),
                ))
        )
}
