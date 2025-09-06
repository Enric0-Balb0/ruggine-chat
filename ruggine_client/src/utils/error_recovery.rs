use crate::components::use_toast;
use leptos::*;

/// Configuration for retry strategies
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_backoff: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            exponential_backoff: true,
        }
    }
}

/// Enhanced error recovery utilities for robust network operations
pub struct ErrorRecovery;

impl ErrorRecovery {
    /// Execute an async operation with retry logic and user feedback
    pub async fn with_retry<F, T, E>(
        operation: F,
        config: RetryConfig,
        operation_name: &str,
    ) -> Result<T, E>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>>>> + Clone,
        E: std::fmt::Debug + Clone,
    {
        let mut last_error = None;
        
        for attempt in 1..=config.max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        Self::show_recovery_success(operation_name);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = Some(e);
                    
                    if attempt < config.max_attempts {
                        let delay = Self::calculate_delay(attempt, &config);
                        Self::show_retry_message(operation_name, attempt, config.max_attempts);
                        crate::utils::timers::sleep_ms(delay).await;
                    }
                }
            }
        }
        
        Self::show_final_error(operation_name);
        Err(last_error.unwrap())
    }
    
    /// Execute an operation with simple error logging (no retry)
    pub async fn with_error_handling<F, T, E>(
        operation: F,
        operation_name: &str,
        show_user_error: bool,
    ) -> Option<T>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        match operation.await {
            Ok(result) => Some(result),
            Err(e) => {
                leptos::logging::error!("Error in {}: {:?}", operation_name, e);
                
                if show_user_error {
                    Self::show_error_toast(operation_name);
                }
                
                None
            }
        }
    }
    
    /// Execute a critical operation that should not fail silently
    pub async fn with_critical_error_handling<F, T, E>(
        operation: F,
        operation_name: &str,
    ) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug + Clone,
    {
        match operation.await {
            Ok(result) => Ok(result),
            Err(e) => {
                leptos::logging::error!("Critical error in {}: {:?}", operation_name, e.clone());
                Self::show_critical_error(operation_name);
                Err(e)
            }
        }
    }
    
    fn calculate_delay(attempt: u32, config: &RetryConfig) -> u64 {
        if config.exponential_backoff {
            let delay = config.base_delay_ms * (2_u64.pow(attempt - 1));
            delay.min(config.max_delay_ms)
        } else {
            config.base_delay_ms
        }
    }
    
    fn show_retry_message(operation: &str, attempt: u32, max_attempts: u32) {
        leptos::logging::warn!("Retrying {} (attempt {}/{})", operation, attempt, max_attempts);
        
        let toast = use_toast();
        toast.info(&format!("Riprovo {}... (tentativo {}/{})", operation, attempt, max_attempts));
    }
    
    fn show_recovery_success(operation: &str) {
        leptos::logging::log!("Successfully recovered: {}", operation);
        
        let toast = use_toast();
        toast.success(&format!("Connessione ripristinata: {}", operation));
    }
    
    fn show_final_error(operation: &str) {
        leptos::logging::error!("Failed to complete after all retries: {}", operation);
        
        let toast = use_toast();
        toast.error(&format!("Impossibile completare {}: verifica la connessione", operation));
    }
    
    fn show_error_toast(operation: &str) {
        let toast = use_toast();
        toast.error(&format!("Errore durante {}: riprova più tardi", operation));
    }
    
    fn show_critical_error(operation: &str) {
        let toast = use_toast();
        toast.error(&format!("Errore critico in {}: contatta il supporto", operation));
    }
}

/// Helper trait for network operations with automatic retry
pub trait NetworkOperation<T, E> {
    fn with_auto_retry(self, operation_name: &str) -> impl std::future::Future<Output = Option<T>>;
    fn with_critical_handling(self, operation_name: &str) -> impl std::future::Future<Output = Result<T, E>>;
}

impl<F, T, E> NetworkOperation<T, E> for F
where
    F: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug + Clone,
{
    async fn with_auto_retry(self, operation_name: &str) -> Option<T> {
        ErrorRecovery::with_error_handling(self, operation_name, true).await
    }
    
    async fn with_critical_handling(self, operation_name: &str) -> Result<T, E> {
        ErrorRecovery::with_critical_error_handling(self, operation_name).await
    }
}
