// Unit tests module - reorganized structure
pub mod services;
pub mod components;
pub mod utils;

// System and data tests
pub mod types_test;
pub mod message_system_test;
pub mod websocket_system_test;

// Hook tests  
pub mod hooks_test;
pub mod use_groups_hook_test;

// Error recovery and resilience tests
pub mod error_recovery_test;
pub mod reconnecting_ws_test;
