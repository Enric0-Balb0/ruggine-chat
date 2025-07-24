use super::auth;
use crate::config::database::Database;
use crate::routes::user;
use crate::state::{auth_state::AuthState, token_state::TokenState, user_state::UserState};
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub fn routes(db_conn: Arc<Database>) -> Router {
    let auth_state = AuthState::new(&db_conn);
    let user_state = UserState::new(&db_conn);
    let token_state = TokenState::new(&db_conn);

    let merged_router = Router::new()
        .nest("/auth", auth::routes().with_state(auth_state))
        .nest("/user", user::routes(user_state, token_state))
        .route("/health", get(|| async { "Healthy..." }));

    let app_router = Router::new()
        .nest("/api", merged_router)
        .layer(TraceLayer::new_for_http());

    app_router
}
