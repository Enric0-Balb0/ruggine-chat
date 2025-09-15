// Test entry point for ruggine_client
// Following same pattern as ruggine_server

mod common;
mod unit;
mod integration;

// Re-export common utilities for use in tests
pub use common::*;
