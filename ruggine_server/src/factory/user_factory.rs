use std::sync::{atomic::{AtomicU32, Ordering}};
use chrono::Utc;

use crate::{dto::user_dto::{UserLoginDto, UserReadDto, UserRegisterDto}, entity::user::{NewUser, User}};

// Global counter for unique test data
static TEST_COUNTER: AtomicU32 = AtomicU32::new(1);

pub struct UserFactory;

impl UserFactory {
    pub fn fake_email(prefix: &str) -> String {
        format!("{}@test.com", prefix)
    }

    pub fn fake_username(prefix: &str) -> String {
        format!("test_{}", prefix)
    }

    pub fn fake_unique_user_login_dto(prefix: &str) -> UserLoginDto {
        UserLoginDto {
            email: format!("{}@test.com", prefix),
            password: "testpass".into(),
        }
    }

    pub fn get_unique_user_information(prefix: &str) -> (String, String, String) {
        let counter: u32 = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
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

    pub fn unique_fake_new_user(prefix: &str, is_active: i8) -> NewUser {
        let (email, username, full_name) = Self::get_unique_user_information(prefix);
        NewUser {
            email,
            user_name: username,
            first_name: full_name.clone(),
            last_name: "Test".to_string(),
            password: "hashed_password".to_string(), // Placeholder for hashed password
            is_active
        }
    }

    pub fn fake_new_user() -> NewUser {
        NewUser {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
            email: "john.doe@example.com".to_string(),
            password: "hashed_password".to_string(),
            is_active: 1,
        }
    }

    pub fn fake_user() -> User {
        User {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
            email: "john.doe@example.com".to_string(),
            password: "hashed_password".to_string(),
            created_at: Utc::now(),
            updated_at: None,
            is_active: 1,
        }
    }

    pub fn fake_read_user_dto() -> UserReadDto {
        UserReadDto {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
            email: "john.doe@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: None,
            is_active: 1,
        }
    }

    pub fn fake_user_register_dto() -> UserRegisterDto {
        UserRegisterDto {
            email: "john.doe@example.com".to_string(),
            password: "securepassword".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            user_name: "johndoe".to_string(),
        }
    }

    pub fn unique_fake_user_register_dto(prefix: &str) -> UserRegisterDto {
        let (email, username, full_name) = Self::get_unique_user_information(prefix);
        UserRegisterDto {
            email,
            password: "securepassword123".to_string(),
            first_name: full_name,
            last_name: "Test".to_string(),
            user_name: username,
        }
    }

}