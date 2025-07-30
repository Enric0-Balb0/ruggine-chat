use std::sync::{atomic::{AtomicU32, Ordering}};
use chrono::{Utc, NaiveDate};

use crate::{dto::user_dto::{UserLoginDto, UserReadDto, UserRegisterDto, ProfileUpdateDto}, entity::user::{NewUser, User, UserStatus, CurrentAction, Gender}};

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

    pub fn unique_fake_new_user(prefix: &str, user_status: UserStatus) -> NewUser {
        let (email, username, full_name) = Self::get_unique_user_information(prefix);
        NewUser {
            email,
            username,
            first_name: full_name.clone(),
            last_name: "Test".to_string(),
            password: "hashed_password".to_string(), // Placeholder for hashed password
            user_status,
            user_type: Default::default(), // Default user type
            birthday: NaiveDate::from_ymd_opt(1992, 5, 15).unwrap(),
            address: format!("{}_{}", prefix, "456 Oak Ave"),
            gender: Gender::Male,
        }
    }

    pub fn unique_fake_user(prefix: &str, user_status: UserStatus) -> User {
        let (email, username, full_name) = Self::get_unique_user_information(prefix);
        let counter: u32 = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let now = Utc::now();
        User {
            id: counter as i32,
            email,
            username,
            first_name: full_name.clone(),
            last_name: "Test".to_string(),
            password: "hashed_password".to_string(), // Placeholder for hashed password
            user_status,
            user_type: Default::default(), // Default user type
            birthday: NaiveDate::from_ymd_opt(1992, 5, 15).unwrap(),
            address: format!("{}_{}", prefix, "456 Oak Ave"),
            gender: Gender::Male,
            created_at: now,
            updated_at: now,
            is_online: Default::default(),
            current_action: Default::default(),
        }
    }

    pub fn fake_new_user() -> NewUser {
        NewUser {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "john.doe@example.com".to_string(),
            password: "hashed_password".to_string(),
            user_status: Default::default(),
            user_type: Default::default(), // Default user type
            birthday: NaiveDate::from_ymd_opt(1992, 5, 15).unwrap(),
            address: "456 Oak Ave".to_string(),
            gender: Gender::Male,
        }
    }

    pub fn fake_user() -> User {
        User {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "john.doe@example.com".to_string(),
            password: "hashed_password".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_status: Default::default(),
            user_type: Default::default(), // Default user type
            birthday: NaiveDate::from_ymd_opt(1992, 5, 15).unwrap(),
            is_online: true,
            address: "456 Oak Ave".to_string(),
            current_action: CurrentAction::Writing,
            gender: Gender::Female,
        }
    }

    pub fn fake_read_user_dto() -> UserReadDto {
        UserReadDto {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            email: "john.doe@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            user_status: Default::default(),
            user_type: Default::default(), // Default user type
            birthday: NaiveDate::from_ymd_opt(1992, 5, 15).unwrap(),
            is_online: true,
            address: "456 Oak Ave".to_string(),
            current_action: CurrentAction::Writing,
            gender: Gender::Female,
        }
    }

    pub fn fake_user_register_dto() -> UserRegisterDto {
        UserRegisterDto {
            email: "john.doe@example.com".to_string(),
            password: "securepassword".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1992, 5, 15).unwrap(),
            address: "456 Oak Ave".to_string(),
            gender: Gender::Male,
        }
    }

    pub fn unique_fake_user_register_dto(prefix: &str) -> UserRegisterDto {
        let (email, username, full_name) = Self::get_unique_user_information(prefix);
        UserRegisterDto {
            email,
            password: "securepassword123".to_string(),
            first_name: full_name,
            last_name: "Test".to_string(),
            username,
            birthday: NaiveDate::from_ymd_opt(1992, 5, 15).unwrap(),
            address: format!("{}_{}", prefix, "456 Oak Ave"),
            gender: Gender::Male,
        }
    }

    pub fn fake_user_update_dto_from_user_read_dto(dto: &UserReadDto, prefix: &str) -> ProfileUpdateDto {
        ProfileUpdateDto {
            first_name: Some(format!("{}_{}", prefix, dto.first_name)),
            last_name: Some(format!("{}_{}", prefix, dto.last_name)),
            birthday: Some(dto.birthday.succ_opt().unwrap_or(dto.birthday)),
            gender: Some(if dto.gender == Gender::Male { Gender::Female } else { Gender::Male }),
            address: Some(format!("{}_{}", prefix, dto.address)),
        }
    }

    pub fn fake_user_update_dto() -> ProfileUpdateDto {
        ProfileUpdateDto {
            first_name: Some("JohnUpdate".to_string()),
            last_name: Some("DoeUpdate".to_string()),
            birthday: Some(NaiveDate::from_ymd_opt(1995, 3, 20).unwrap()),
            address: Some("465 Oak Ave Update".to_string()),
            gender: Some(Gender::Female),
        }
    }

    pub fn fake_user_update_dto_partial() -> ProfileUpdateDto {
        ProfileUpdateDto {
            first_name: Some("JohnUpdate".to_string()),
            last_name: None,
            birthday: None,
            address: Some("465 Oak Ave Update".to_string()),
            gender: None,
        }
    }

    pub fn fake_user_update_dto_empty() -> ProfileUpdateDto {
        ProfileUpdateDto {
            first_name: None,
            last_name: None,
            birthday: None,
            address: None,
            gender: None,
        }
    }

    pub fn unique_fake_user_update_dto(prefix: &str) -> ProfileUpdateDto {
        let counter: u32 = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        ProfileUpdateDto {
            first_name: Some(format!("Updated{}{}", prefix, counter)),
            last_name: Some(format!("UpdatedLast{}", counter)),
            birthday: Some(NaiveDate::from_ymd_opt(1985, 8, 10).unwrap()),
            address: Some(format!("Updated Address {} {}", prefix, counter)),
            gender: Some(Gender::Other),
        }
    }

}