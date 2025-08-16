pub mod create;
pub mod find_by_group_chat_id;

use axum::Router;
use crate::entity::user::all_user_types;

pub fn routes(
    text_message_state: crate::state::text_message_state::TextMessageState,
    token_state: crate::state::token_state::TokenState,
) -> Router {
    use tower::ServiceBuilder;
    use axum::middleware;

    Router::new()
        .merge(
            create::routes()
                .with_state(text_message_state.clone())
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(
                        token_state.clone(),
                        crate::middleware::auth_middleware::auth(all_user_types()),
                    ),
                ))
        )
        .merge(
            find_by_group_chat_id::routes()
                .with_state(text_message_state.clone())
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(
                        token_state,
                        crate::middleware::auth_middleware::auth(all_user_types()),
                    ),
                ))
        )
}
