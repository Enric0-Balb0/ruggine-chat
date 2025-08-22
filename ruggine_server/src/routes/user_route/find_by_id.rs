use crate::handler::user_handler::find_by_id_handler::find_by_id;
use crate::state::user_state::UserState;
use axum::{routing::get, Router};

pub fn routes() -> Router<UserState> {
    Router::new().route("/:id", get(find_by_id))
}
