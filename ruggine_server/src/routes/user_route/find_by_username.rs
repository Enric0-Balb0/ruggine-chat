use crate::handler::user_handler::find_by_username_handler::find_by_username;
use crate::state::user_state::UserState;
use axum::{routing::get, Router};

pub fn routes() -> Router<UserState> {
    Router::new().route("/username/:username", get(find_by_username))
}
