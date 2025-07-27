use crate::handler::auth_handler::login_handler;
use crate::state::auth_state::AuthState;
use axum::{routing::post, Router};

pub fn routes() -> Router<AuthState> {
    Router::new().route("/login", post(login_handler::login))
}
