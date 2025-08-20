use crate::response::api_response::ApiErrorResponse;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::Request;
use axum::extract::{rejection::JsonRejection, FromRequest};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::de::DeserializeOwned;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use validator::Validate;
use crate::websocket::message::{ControlMessage, WsError};
use crate::websocket::WebSocketMessage;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedRequest<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S, Body> for ValidatedRequest<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = RequestError;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedRequest(value))
    }
}

impl IntoResponse for RequestError {
    fn into_response(self) -> Response {
        match self {
            RequestError::ValidationError(_) => {
                ApiErrorResponse::send(400, Some(self.to_string().replace('\n', ", ")))
            }
            RequestError::JsonRejection(_) => ApiErrorResponse::send(400, Some(self.to_string())),
        }
    }
}

pub struct ValidatedWebSocketMessage(pub WebSocketMessage);

impl ValidatedWebSocketMessage {
    /// Tenta di creare un messaggio validato dal testo JSON
    /// In caso di errore, invia direttamente al client l'errore e restituisce Err
    pub async fn from_text_or_error(
        text: &str,
        tx: &UnboundedSender<WebSocketMessage>,
    ) -> Result<Self, ()> {
        match WebSocketMessage::from_json(text) {
            Ok(msg) => Ok(Self(msg)),
            Err(e) => {
                let ws_error: WsError = WsError {
                    code: 422,
                    message: format!("Invalid WebSocket message: {}", e),
                };
                let _ = tx.send(WebSocketMessage::Control(ControlMessage::ValidationError {
                    code: ws_error.code,
                    message: ws_error.message.clone(),
                    message_id: None,
                }));
                Err(())
            }
        }
    }
}