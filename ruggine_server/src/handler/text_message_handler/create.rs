use crate::dto::text_message_dto::{TextMessageCreateDto, TextMessageReadDto};
use crate::error::{api_error::ApiError, request_error::ValidatedRequest};
use crate::response::api_response::ApiSuccessResponse;
use crate::routes::websocket::group;
use crate::service::websocket::group_service;
use crate::state::text_message_state::TextMessageState;
use axum::response::sse::Event;
use axum::{extract::State, Extension, Json};
use chrono::Utc;
use crate::entity::user::User;
use tracing::{info, warn};
use crate::websocket::GroupEvent::NewMessage;
use crate::websocket::{GroupEvent, ServerEvent, WebSocketMessage};

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
        .create(payload.clone(), current_user.id)
        .await?;
    
    // Invia notifica WebSocket ai membri del gruppo se il servizio è disponibile
    if let Some(websocket_service) = &state.websocket_group_service {
        let notification = WebSocketMessage::Event {
            event: ServerEvent::Groups(GroupEvent::NewMessage {
                message_id: text_message.id,
                group_id: payload.group_chat_id,
                sender_id: current_user.id,
                sender_username: current_user.username.clone(),
                content: text_message.content.clone(),
                sent_at: text_message.sent_at,
            }),
            timestamp: Utc::now(),
        };
        
        crate::handler::websocket::group_handler::handle_new_group_message(
            state.websocket_group_service.clone().unwrap().clone(),
            state.ws_manager.clone().unwrap().clone(),
            payload.group_chat_id,
            notification
        ).await;
    } else {
        warn!("WebSocket group service not available for sending notifications");
    }
    
    Ok(Json(ApiSuccessResponse::send(text_message)))
}