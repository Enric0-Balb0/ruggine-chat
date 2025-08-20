use crate::{ websocket::message::WsError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Connection not found")]
    ConnectionNotFound,
    #[error("Connection not active")]
    ConnectionNotActive,
    #[error("Connection timed out")]
    ConnectionTimedOut,
    #[error("Connection closed")]
    ConnectionClosed,
    #[error("Failed to send message")]
    FailedToSendMessage,
}

impl From<ConnectionError> for WsError {
    fn from(err: ConnectionError) -> Self {
        match err {
            ConnectionError::ConnectionNotFound => WsError { code: 404, message: err.to_string() },
            ConnectionError::ConnectionNotActive => WsError { code: 400, message: err.to_string() },
            ConnectionError::ConnectionTimedOut => WsError { code: 408, message: err.to_string() },
            ConnectionError::ConnectionClosed => WsError { code: 410, message: err.to_string() },
            ConnectionError::FailedToSendMessage => WsError { code: 500, message: err.to_string() },
        }
    }
}
