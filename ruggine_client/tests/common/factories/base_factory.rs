// Base factory utilities for generating consistent test data

use std::sync::atomic::{AtomicU32, Ordering};

static TEST_COUNTER: AtomicU32 = AtomicU32::new(1);

pub struct BaseFactory;

impl BaseFactory {
    /// Generate unique identifier for tests
    pub fn get_unique_id() -> u32 {
        TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
    }
    
    /// Generate unique user information for tests
    pub fn get_unique_user_info(prefix: &str) -> (String, String, String) {
        let counter = Self::get_unique_id();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        let unique_suffix = format!("{}_{}", counter, timestamp);
        let email = format!("{}{}@test.com", prefix, unique_suffix);
        let username = format!("{}_{}", prefix, unique_suffix);
        let full_name = format!("{}{}", prefix, unique_suffix);
        
        (email, username, full_name)
    }
    
    /// Generate unique group information for tests
    pub fn get_unique_group_info(prefix: &str) -> (String, String) {
        let counter = Self::get_unique_id();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        let unique_suffix = format!("{}_{}", counter, timestamp);
        let name = format!("{}_group_{}", prefix, unique_suffix);
        let description = format!("{} test group {}", prefix, unique_suffix);
        
        (name, description)
    }
    
    /// Generate unique token for tests
    pub fn get_unique_token() -> String {
        let unique_id = chrono::Utc::now().timestamp_nanos();
        format!("test_token_{}", unique_id)
    }
    
    /// Generate unique message content
    pub fn get_unique_message_content(prefix: &str) -> String {
        let counter = Self::get_unique_id();
        format!("{} message content {}", prefix, counter)
    }
    
    /// Generate unique UUID-like string for tests
    pub fn generate_uuid() -> String {
        let unique_id = chrono::Utc::now().timestamp_nanos();
        format!("test-uuid-{}", unique_id)
    }
}
