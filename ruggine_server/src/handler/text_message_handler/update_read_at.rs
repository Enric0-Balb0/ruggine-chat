use crate::dto::user_dto::{UserReadDto};
use crate::entity::user::{User};
use crate::error::{api_error::ApiError, request_error::ValidatedRequest};
use crate::response::api_response::ApiSuccessResponse;
use axum::{extract::State, Extension, Json};
use tracing::warn;
use validator::Validate;
use crate::dto::text_message_dto::{TextMessageInfoReadAtDtoUpdate, TextMessageInfoReadDto};
use crate::state::text_message_state::TextMessageState;

#[utoipa::path(
    patch,
    path = "/api/text_message/update_read_at",
    request_body = TextMessageInfoReadAtDtoUpdate,
    responses(
        (status = 200, description = "Text message updated successfully", body = ApiSuccessResponseUserReadDto),
        (status = 400, description = "Bad request: Message read at already updated or invalid"),
        (status = 403, description = "User cannot access message"),
        (status = 404, description = "Text message, group chat or text message infos not found for the user"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "TextMessage"
)]
pub async fn update_read_at(
    Extension(current_user): Extension<User>,
    State(state): State<TextMessageState>,
    ValidatedRequest(payload): ValidatedRequest<TextMessageInfoReadAtDtoUpdate>,
) -> Result<Json<ApiSuccessResponse<TextMessageInfoReadDto>>, ApiError> {
    // Update the user profile
    let updated_message = state.text_message_service.update_read_at(current_user.id, payload).await?;

    // Invia notifica WebSocket ai membri del gruppo se il servizio è disponibile
    if let Some(websocket_service) = &state.websocket_group_service {
        let group_id = state.text_message_service.find_by_id(updated_message.text_message_id).await?.group_chat_id;
        crate::handler::websocket::chat_handler::handle_new_read_text_message(
            websocket_service.clone(),
            state.ws_manager.clone().unwrap().clone(),
            updated_message.clone(),
            group_id,
        ).await;
    } else {
        warn!("WebSocket group service not available for sending notifications");
    }

    Ok(Json(ApiSuccessResponse::send(updated_message)))
}
