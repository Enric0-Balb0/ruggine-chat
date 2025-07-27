use crate::handler::user_handler::profile_handler;
use axum::{routing::get, Router};

pub fn routes() -> Router {
    Router::new()
        .route("/profile", get(profile_handler::profile))           // GET /user/profile
        //.route("/:id", get(profile_handler::get_user_by_id))        // GET /user/:id
}
