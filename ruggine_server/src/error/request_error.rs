use crate::response::api_response::ApiErrorResponse;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::Request;
use axum::extract::{rejection::JsonRejection, FromRequest, FromRequestParts, Query};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::de::DeserializeOwned;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;
use utoipa::IntoParams;
use validator::{Validate, ValidationError, ValidationErrors};
use crate::error::api_error::ApiError;
use crate::error::db_error::DbError;
use crate::websocket::message::{ControlMessage, WsError};
use crate::websocket::WebSocketMessage;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error(transparent)]
    ValidationError(ValidationErrors),
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),
    #[error(transparent)]
    QueryValidationError(ValidationErrors),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedRequest<T>(pub T);

// per JSON body
#[async_trait]
impl<T, S> FromRequest<S, Body> for ValidatedRequest<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = RequestError;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate().map_err(RequestError::ValidationError)?;
        Ok(ValidatedRequest(value))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

#[async_trait::async_trait]
impl<T, S> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = RequestError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Estrai la query dai parametri
        let Query(query): Query<T> = Query::from_request_parts(parts, _state)
            .await
            .map_err(|err| {
                // costruisci un ValidationErrors vuoto
                let mut errors = ValidationErrors::new();

                // crea un ValidationError con dentro il messaggio originale
                let mut ve = ValidationError::new("query_deserialization");
                ve.message = Some(err.to_string().into());

                // aggiungilo come errore "globale", non legato a nessun campo
                errors.add("", ve);

                RequestError::QueryValidationError(errors)
            })?;

        // Applica la validazione validator
        if let Err(errors) = query.validate() {
            eprintln!("Validation errors: {:?}", errors);
            return Err(RequestError::QueryValidationError(errors));
        }

        Ok(ValidatedQuery(query))
    }
}
impl IntoResponse for RequestError {
    fn into_response(self) -> Response {
        match self {
            RequestError::ValidationError(_) => {
                ApiErrorResponse::send(400, Some(self.to_string().replace('\n', ", ")))
            }
            RequestError::QueryValidationError(_) => {
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