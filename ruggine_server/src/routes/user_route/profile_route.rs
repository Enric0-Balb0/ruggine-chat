use crate::handler::user_handler::{profile_handler, update_profile_handler};
use crate::state::user_state::UserState;
use axum::{routing::{get, patch}, Router};

pub fn routes() -> Router<UserState> {
    Router::new()
        .route("/profile", get(profile_handler::profile))           // GET /user/profile
        .route("/profile", patch(update_profile_handler::update_profile))  // PATCH /user/profile
        //.route("/:id", get(profile_handler::get_user_by_id))        // GET /user/:id
}
