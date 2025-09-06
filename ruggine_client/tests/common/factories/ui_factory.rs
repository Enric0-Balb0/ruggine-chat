// UI components test data factory

use super::base_factory::BaseFactory;
use ruggine_client_ui::types::common::LoadingState;

pub struct UIFactory;

impl UIFactory {
    /// Create mock toast message
    pub fn mock_success_toast() -> (String, String) {
        ("success".to_string(), "Operation completed successfully".to_string())
    }
    
    /// Create mock error toast
    pub fn mock_error_toast() -> (String, String) {
        ("error".to_string(), "An error occurred".to_string())
    }
    
    /// Create mock info toast
    pub fn mock_info_toast() -> (String, String) {
        ("info".to_string(), "Information message".to_string())
    }
    
    /// Create mock warning toast
    pub fn mock_warning_toast() -> (String, String) {
        ("warning".to_string(), "Warning message".to_string())
    }
    
    /// Create LoadingState::Idle
    pub fn idle_loading_state<T>() -> LoadingState<T> {
        LoadingState::Idle
    }
    
    /// Create LoadingState::Loading
    pub fn loading_state<T>() -> LoadingState<T> {
        LoadingState::Loading
    }
    
    /// Create LoadingState::Success with data
    pub fn success_loading_state<T>(data: T) -> LoadingState<T> {
        LoadingState::Success(data)
    }
    
    /// Create LoadingState::Error
    pub fn error_loading_state<T>(error: &str) -> LoadingState<T> {
        LoadingState::Error(error.to_string())
    }
    
    /// Create mock icon props
    pub fn mock_icon_props() -> (String, u32, String) {
        let icon_name = "home".to_string();
        let size = 24u32;
        let class = "icon-class".to_string();
        (icon_name, size, class)
    }
    
    /// Create mock modal state
    pub fn mock_modal_state() -> (bool, String, String) {
        let is_open = true;
        let title = "Test Modal".to_string();
        let content = "This is a test modal content".to_string();
        (is_open, title, content)
    }
    
    /// Create mock theme state
    pub fn mock_theme_state() -> String {
        "light".to_string()
    }
    
    /// Create dark theme state
    pub fn dark_theme_state() -> String {
        "dark".to_string()
    }
    
    /// Create mock chat component props
    pub fn mock_chat_props() -> (i32, String, bool) {
        let group_id = 1;
        let group_name = "Test Group".to_string();
        let is_active = true;
        (group_id, group_name, is_active)
    }
    
    /// Create mock layout props
    pub fn mock_layout_props() -> (String, bool, Vec<String>) {
        let title = "Test Page".to_string();
        let show_sidebar = true;
        let breadcrumbs = vec!["Home".to_string(), "Test".to_string()];
        (title, show_sidebar, breadcrumbs)
    }
    
    /// Create unique UI identifier
    pub fn unique_ui_id() -> String {
        format!("ui_element_{}", BaseFactory::get_unique_id())
    }
}
