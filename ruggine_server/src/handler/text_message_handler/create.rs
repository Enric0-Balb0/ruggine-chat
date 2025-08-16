use crate::dto::text_message_dto::{TextMessageCreateDto, TextMessageReadDto};
use crate::error::{api_error::ApiError, request_error::ValidatedRequest};
use crate::response::api_response::ApiSuccessResponse;
use crate::state::text_message_state::TextMessageState;
use axum::{extract::State, Extension, Json};
use crate::entity::user::User;

#[utoipa::path(
    post,
    path = "/api/text_message/create",
    request_body = TextMessageCreateDto,
    responses(
        (status = 200, description = "Text message created successfully", body = ApiSuccessResponseTextMessageReadDto),
        (status = 400, description = "Invalid request data"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "User cannot access messages in this group"),
        (status = 422, description = "Validation error"),
        (status = 500, description = "Internal server error")
    ),
    tag = "TextMessage",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create(
    Extension(current_user): Extension<User>,
    State(state): State<TextMessageState>,
    ValidatedRequest(payload): ValidatedRequest<TextMessageCreateDto>,
) -> Result<Json<ApiSuccessResponse<TextMessageReadDto>>, ApiError> {
    let text_message = state
        .text_message_service
        .create(payload, current_user.id)
        .await?;
    
    Ok(Json(ApiSuccessResponse::send(text_message)))
}