pub mod group;

use axum::Router;
use tower::ServiceBuilder;
use axum::middleware;

use crate::{
    state::websocket::WebSocketState,
    entity::user::all_user_types,
};

/// Routes per WebSocket organizzate in moduli separati
pub fn routes(state: WebSocketState) -> Router<()> {
    // Endpoint WebSocket senza autenticazione middleware (gestisce token manualmente)
    let websocket_routes = Router::new()
        .merge(group::routes())
        .with_state(state.clone())
        .layer(ServiceBuilder::new().layer(
            middleware::from_fn_with_state(
                state.token_state.as_ref().clone(),
                crate::middleware::auth_middleware::auth(all_user_types())
            ),
        ));

    websocket_routes
}