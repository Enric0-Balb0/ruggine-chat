use crate::response::api_response::ApiErrorResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextMessageError {
    #[error("Message not found")]
    MessageNotFound,
}

impl IntoResponse for TextMessageError {
    fn into_response(self) -> Response {
        let status_code = match self {
            TextMessageError::MessageNotFound => StatusCode::NOT_FOUND,
        };

        ApiErrorResponse::send(status_code.as_u16(), Some(self.to_string()))
    }
}
