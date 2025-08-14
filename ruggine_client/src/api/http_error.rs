use thiserror::Error;

/// Errori per le chiamate HTTP
#[derive(Error, Debug, Clone)]
pub enum HttpError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String },
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden")]
    Forbidden,
    
    #[error("Not found")]
    NotFound,
    
    #[error("Server error: {0}")]
    Server(String),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for HttpError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            HttpError::Timeout
        } else if error.is_request() {
            HttpError::Network(error.to_string())
        } else if let Some(status) = error.status() {
            match status.as_u16() {
                401 => HttpError::Unauthorized,
                403 => HttpError::Forbidden,
                404 => HttpError::NotFound,
                500..=599 => HttpError::Server(error.to_string()),
                _ => HttpError::Http {
                    status: status.as_u16(),
                    message: error.to_string(),
                },
            }
        } else {
            HttpError::Unknown(error.to_string())
        }
    }
}

impl From<serde_json::Error> for HttpError {
    fn from(error: serde_json::Error) -> Self {
        HttpError::Serialization(error.to_string())
    }
}
