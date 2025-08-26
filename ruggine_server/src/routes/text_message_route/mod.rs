pub mod create;
pub mod find_by_group_chat_id;
mod find_messages_not_sent_yet;
mod find_messages_not_read_yet;
mod update_read_at;

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
            find_messages_not_sent_yet::routes()
                .with_state(text_message_state.clone())
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(
                        token_state.clone(),
                        crate::middleware::auth_middleware::auth(all_user_types()),
                    ),
                ))
        )
        .merge(
            find_messages_not_read_yet::routes()
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
                        token_state.clone(),
                        crate::middleware::auth_middleware::auth(all_user_types()),
                    ),
                ))
        )
        .merge(
            update_read_at::routes()
                .with_state(text_message_state.clone())
                .layer(ServiceBuilder::new().layer(
                    middleware::from_fn_with_state(
                        token_state.clone(),
                        crate::middleware::auth_middleware::auth(all_user_types()),
                    ),
                ))
        )

}
