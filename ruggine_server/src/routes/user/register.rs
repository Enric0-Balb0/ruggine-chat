use crate::handler::user::register;
use crate::state::user_state::UserState;
use axum::{routing::post, Router};

pub fn routes() -> Router<UserState> {
    Router::new().route("/register", post(register::register))
}
