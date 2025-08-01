use serde::{Deserialize, Serialize};

/// Additional wrapper responses from server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSuccessResponseInvitationCreateDto {
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSuccessResponseInvitationUpdateDto {
    pub data: serde_json::Value,
}

// =============================================================================
// CLIENT UTILITIES - Common response types
// =============================================================================

/// Standardized API response from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub success: bool,
    pub message: Option<String>,
}

/// Standardized API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Loading state for async operations
#[derive(Debug, Clone, PartialEq)]
pub enum LoadingState<T> {
    Idle,
    Loading,
    Success(T),
    Error(String),
}

impl<T> LoadingState<T> {
    /// Check if currently loading
    pub fn is_loading(&self) -> bool {
        matches!(self, LoadingState::Loading)
    }

    /// Check if operation succeeded
    pub fn is_success(&self) -> bool {
        matches!(self, LoadingState::Success(_))
    }

    /// Check if operation failed
    pub fn is_error(&self) -> bool {
        matches!(self, LoadingState::Error(_))
    }

    /// Get success data if available
    pub fn data(&self) -> Option<&T> {
        match self {
            LoadingState::Success(data) => Some(data),
            _ => None,
        }
    }

    /// Get error message if available
    pub fn error(&self) -> Option<&String> {
        match self {
            LoadingState::Error(error) => Some(error),
            _ => None,
        }
    }
}

impl<T> Default for LoadingState<T> {
    fn default() -> Self {
        LoadingState::Idle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loading_state_behavior() {
        let idle: LoadingState<String> = LoadingState::Idle;
        let loading: LoadingState<String> = LoadingState::Loading;
        let success = LoadingState::Success("data".to_string());
        let error: LoadingState<String> = LoadingState::Error("error".to_string());

        assert!(!idle.is_loading());
        assert!(loading.is_loading());
        assert!(!success.is_loading());
        assert!(!error.is_loading());

        assert!(!idle.is_success());
        assert!(!loading.is_success());
        assert!(success.is_success());
        assert!(!error.is_success());

        assert!(!idle.is_error());
        assert!(!loading.is_error());
        assert!(!success.is_error());
        assert!(error.is_error());

        assert!(idle.data().is_none());
        assert!(loading.data().is_none());
        assert_eq!(success.data(), Some(&"data".to_string()));
        assert!(error.data().is_none());

        assert!(idle.error().is_none());
        assert!(loading.error().is_none());
        assert!(success.error().is_none());
        assert_eq!(error.error(), Some(&"error".to_string()));
    }

    #[test]
    fn test_loading_state_default() {
        let default_state: LoadingState<i32> = LoadingState::default();
        assert!(matches!(default_state, LoadingState::Idle));
    }
}
