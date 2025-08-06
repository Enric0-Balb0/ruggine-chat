use super::{auth_route, group_chat_route, group_membership_route, invitation_route};
use crate::config::database::Database;
use crate::docs::ApiDoc;
use crate::routes::user_route;
use crate::state::group_membership_state::GroupMembershipState;
use crate::state::invitation_state;
use crate::state::{auth_state::AuthState, token_state::TokenState, user_state::UserState, invitation_state::InvitationState, group_chat_state::GroupChatState};
use axum::routing::get;
use axum::{Json, Router};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn routes(db_conn: Arc<Database>) -> Router {
    let auth_state = AuthState::new(&db_conn);
    let user_state = UserState::new(&db_conn);
    let token_state = TokenState::new(&db_conn);
    let group_chat_state = GroupChatState::new(&db_conn);
    let invitation_state = InvitationState::new(&db_conn);
    let group_membership_state = GroupMembershipState::new(&db_conn);

    let merged_router = Router::new()
        .nest("/auth", auth_route::routes().with_state(auth_state))
        .nest("/user", user_route::routes(user_state, token_state.clone()))
        .nest("/group_chat", group_chat_route::routes(group_chat_state, token_state.clone()))
        .nest("/invitation", invitation_route::routes(
            invitation_state,
            token_state.clone(),
        ))
        .nest("/group_membership", group_membership_route::routes(
            group_membership_state,
            token_state.clone(),
        ))
        .route("/health", get(|| async { "Healthy..." }))
        .route("/api-docs/openapi.json", get(|| async { Json(ApiDoc::openapi()) }));

    let app_router = Router::new()
        .nest("/api", merged_router)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http());

    app_router
}
