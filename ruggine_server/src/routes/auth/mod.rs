pub mod login_route;

use crate::state::auth_state::AuthState;
use axum::Router;

pub fn routes() -> Router<AuthState> {
    login_route::routes()
    // In futuro puoi fare .merge(register::routes()), .merge(refresh::routes()), ecc.
}
