use crate::dto::text_message_pagination_dto::TextMessagePaginationQuery;
use crate::error::api_error::ApiError;
use crate::response::PaginatedTextMessageResponse;
use crate::state::text_message_state::TextMessageState;
use crate::entity::user::User;
use axum::{extract::{Path, Query, State}, Extension, Json};

#[utoipa::path(
    get,
    path = "/api/text_message/group/{group_chat_id}/messages",
    params(
        ("group_chat_id" = i32, Path, description = "Group chat ID to retrieve messages from"),
        TextMessagePaginationQuery
    ),
    responses(
        (status = 200, description = "Text messages retrieved successfully", body = PaginatedTextMessageResponse),
        (status = 404, description = "Group chat not found"),
        (status = 403, description = "User cannot access messages"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 400, description = "Bad request - Invalid pagination parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "TextMessage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn find_by_group_chat_id(
    Extension(current_user): Extension<User>,
    State(state): State<TextMessageState>,
    Path(group_chat_id): Path<i32>,
    Query(pagination_query): Query<TextMessagePaginationQuery>,
) -> Result<Json<PaginatedTextMessageResponse>, ApiError> {
    let messages = state
        .text_message_service
        .find_by_group_chat_id_paginated(group_chat_id, current_user.id, pagination_query)
        .await?;

    Ok(Json(messages))
}
