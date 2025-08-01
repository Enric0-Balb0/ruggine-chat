use crate::http::error::HttpError;

/// Authentication related errors
#[derive(Debug, Clone)]
pub enum AuthError {
    /// Invalid input parameters
    InvalidInput(String),
    /// User not authenticated
    NotAuthenticated,
    /// HTTP request error
    Http(HttpError),
    /// Storage operation error
    Storage(StorageError),
}

/// Storage related errors
#[derive(Debug, Clone)]
pub enum StorageError {
    /// Failed to access storage
    AccessFailed(String),
    /// Invalid data format
    InvalidData(String),
    /// Storage not available
    NotAvailable,
    /// Write operation failed
    WriteError(String),
    /// Serialization error
    Serialization(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AuthError::NotAuthenticated => write!(f, "User not authenticated"),
            AuthError::Http(err) => write!(f, "HTTP error: {}", err),
            AuthError::Storage(err) => write!(f, "Storage error: {}", err),
        }
    }
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::AccessFailed(msg) => write!(f, "Storage access failed: {}", msg),
            StorageError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            StorageError::NotAvailable => write!(f, "Storage not available"),
            StorageError::WriteError(key) => write!(f, "Failed to write key: {}", key),
            StorageError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}
impl std::error::Error for StorageError {}

impl From<HttpError> for AuthError {
    fn from(error: HttpError) -> Self {
        AuthError::Http(error)
    }
}

impl From<StorageError> for AuthError {
    fn from(error: StorageError) -> Self {
        AuthError::Storage(error)
    }
}
