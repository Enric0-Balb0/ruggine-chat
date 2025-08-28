use crate::entity::user::{User, UserStatus, UserType, Gender};
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;
use utoipa::ToSchema;

#[derive(Clone, Serialize, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "email": "user@example.com",
    "password": "securepassword123"
}))]
pub struct UserLoginDto {
    #[validate(email(message = "Email is not valid"))]
    #[schema(example = "user@example.com")]
    pub email: String,
    #[validate(length(
        min = 1,
        max = 256,
        message = "Password must be between 1 and 256 characters"
    ))]
    #[schema(example = "securepassword123")]
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize, Validate, PartialEq, Eq, ToSchema)]
#[schema(example = json!({
    "email": "newuser@example.com",
    "password": "securepassword123",
    "first_name": "John",
    "last_name": "Doe",
    "username": "johndoe",
    "birthday": "1990-01-01",
    "address": "123 Main St",
    "gender": "male"
}))]
pub struct UserRegisterDto {
    #[validate(email(message = "Email is not valid"))]
    #[schema(example = "newuser@example.com")]
    pub email: String,
    #[validate(length(
        min = 1,
        max = 256,
        message = "Password must be between 1 and 256 characters"
    ))]
    #[schema(example = "securepassword123")]
    pub password: String,
    #[validate(length(
        min = 1,
        max = 256,
        message = "Last name must be between 1 and 256 characters"
    ))]
    #[schema(example = "John")]
    pub first_name: String,
    #[validate(length(
        min = 1,
        max = 256,
        message = "Last name must be between 1 and 256 characters"
    ))]
    #[schema(example = "Doe")]
    pub last_name: String,
    #[validate(length(
        min = 1,
        max = 256,
        message = "Username must be between 1 and 256 characters"
    ))]
    #[schema(example = "johndoe")]
    pub username: String,
    #[schema(example = "1990-01-01")]
    pub birthday: NaiveDate,
    #[validate(length(
        min = 1,
        max = 256,
        message = "Address must be between 1 and 256 characters"
    ))]
    #[schema(example = "123 Main St")]
    pub address: String,
    #[schema(example = "male")]
    pub gender: Gender,
}

#[derive(Clone, Serialize, Deserialize, Validate, PartialEq, Eq, ToSchema)]
#[schema(example = json!({
    "first_name": "NewJohn",
    "last_name": "NewDoe",
    "birthday": "2000-01-01",
    "address": "123 New Main St",
    "gender": "other"
}))]
pub struct ProfileUpdateDto {
    #[validate(length(
        min = 1,
        max = 256,
        message = "First name must be between 1 and 256 characters"
    ))]
    #[schema(example = "John")]
    pub first_name: Option<String>,
    #[validate(length(
        min = 1,
        max = 256,
        message = "Last name must be between 1 and 256 characters"
    ))]
    #[schema(example = "Doe")]
    pub last_name: Option<String>,
    #[schema(example = "1990-01-01")]
    pub birthday: Option<NaiveDate>,
    #[validate(length(
        min = 1,
        max = 256,
        message = "Address must be between 1 and 256 characters"
    ))]
    #[schema(example = "123 Main St")]
    pub address: Option<String>,
    #[schema(example = "male")]
    pub gender: Option<Gender>,
}

#[derive(Clone, Serialize, Deserialize, Validate, PartialEq, Eq, ToSchema)]
pub struct UpdateOnlineDto {
    #[schema(example = "false")]
    pub online: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, ToSchema)]
#[schema(example = json!({
    "id": 1,
    "first_name": "John",
    "last_name": "Doe",
    "username": "johndoe",
    "email": "user@example.com",
    "created_at": "2023-01-01T00:00:00Z",
    "updated_at": "2023-01-01T00:00:00Z",
    "user_status": "active",
    "user_type": "end_user",
    "birthday": "1990-01-01",
    "is_online": false,
    "address": "123 Main St",
    "gender": "male"
}))]
pub struct UserReadDto {
    #[schema(example = 1)]
    pub id: i32,
    #[schema(example = "John")]
    pub first_name: String,
    #[schema(example = "Doe")]
    pub last_name: String,
    #[schema(example = "johndoe")]
    pub username: String,
    #[schema(example = "user@example.com")]
    pub email: String,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
    #[schema(example = "active")]
    pub user_status: UserStatus,
    #[schema(example = "end_user")]
    pub user_type: UserType,
    #[schema(example = "1990-01-01")]
    pub birthday: NaiveDate,
    #[schema(example = false)]
    pub is_online: bool,
    #[schema(example = "123 Main St")]
    pub address: String,
    #[schema(example = "male")]
    pub gender: Gender,
}

impl UserReadDto {
    // TODO: Add test for from
    pub fn from(model: User) -> UserReadDto {
        Self {
            id: model.id,
            first_name: model.first_name,
            last_name: model.last_name,
            username: model.username,
            email: model.email,
            created_at: model.created_at,
            updated_at: model.updated_at,
            user_status: model.user_status,
            user_type: model.user_type,
            birthday: model.birthday,
            is_online: model.is_online,
            address: model.address,
            gender: model.gender,
        }
    }
}

impl std::fmt::Debug for UserLoginDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("email", &self.email)
            .finish()
    }
}

impl std::fmt::Debug for UserRegisterDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("first_name", &self.first_name)
            .field("last_name", &self.last_name)
            .field("username", &self.username)
            .field("email", &self.email)
            .finish()
    }
}

impl std::fmt::Debug for ProfileUpdateDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserUpdate")
            .field("first_name", &self.first_name)
            .field("last_name", &self.last_name)
            .field("birthday", &self.birthday)
            .field("address", &self.address)
            .field("gender", &self.gender)
            .finish()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use validator::Validate;

    #[test]
    fn test_user_login_dto_valid() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        assert!(login_dto.validate().is_ok());
        assert_eq!(login_dto.email, "test@example.com");
        assert_eq!(login_dto.password, "password123");
    }

    #[test]
    fn test_user_login_dto_invalid_email() {
        let login_dto = UserLoginDto {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
        };

        let validation_result = login_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("email"));
    }

    #[test]
    fn test_user_login_dto_password_too_short() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "".to_string(), // Less than 1 characters
        };

        let validation_result = login_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("password"));
    }

    #[test]
    fn test_user_login_dto_password_too_long() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "a".repeat(257), // More than 256 characters
        };

        let validation_result = login_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("password"));
    }

    #[test]
    fn test_user_register_dto_valid() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Main St".to_string(),
            gender: Gender::Male,
        };

        assert!(register_dto.validate().is_ok());
        assert_eq!(register_dto.email, "test@example.com");
        assert_eq!(register_dto.password, "password123");
        assert_eq!(register_dto.first_name, "John");
        assert_eq!(register_dto.last_name, "Doe");
        assert_eq!(register_dto.username, "johndoe");
        assert_eq!(register_dto.birthday, NaiveDate::from_ymd_opt(1990, 1, 1).unwrap());
        assert_eq!(register_dto.address, "123 Main St");
        assert_eq!(register_dto.gender, Gender::Male);
    }

    #[test]
    fn test_user_register_dto_invalid_email() {
        let register_dto = UserRegisterDto {
            email: "invalid-email".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Main St".to_string(),
            gender: Gender::Male,
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("email"));
    }

    #[test]
    fn test_user_register_dto_password_too_short() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "".to_string(), // Less than 1 characters
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Main St".to_string(),
            gender: Gender::Male,
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("password"));
    }

    #[test]
    fn test_user_register_dto_password_too_long() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "a".repeat(257), // More than 256 characters
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Main St".to_string(),
            gender: Gender::Male,
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("password"));
    }

    #[test]
    fn test_user_register_dto_username_too_short() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "".to_string(), // Less than 1 character
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Main St".to_string(),
            gender: Gender::Male,
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("username"));
    }

    #[test]
    fn test_user_register_dto_username_too_long() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "a".repeat(257), // More than 256 characters
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Main St".to_string(),
            gender: Gender::Male,
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("username"));
    }

    #[test]
    fn test_user_login_dto_debug_format() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let debug_string = format!("{:?}", login_dto);
        assert!(debug_string.contains("test@example.com"));
        // Password should not be shown in debug format for security
        assert!(!debug_string.contains("password123"));
    }

    #[test]
    fn test_user_register_dto_debug_format() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Main St".to_string(),
            gender: Gender::Male,
        };

        let debug_string = format!("{:?}", register_dto);
        assert!(debug_string.contains("John"));
        assert!(debug_string.contains("Doe"));
        assert!(debug_string.contains("johndoe"));
        assert!(debug_string.contains("test@example.com"));
        // Password should not be shown in debug format for security
        assert!(!debug_string.contains("password123"));
    }

    #[test]
    fn test_user_login_dto_clone() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let cloned_dto = login_dto.clone();
        assert_eq!(login_dto.email, cloned_dto.email);
        assert_eq!(login_dto.password, cloned_dto.password);
    }

    #[test]
    fn test_user_register_dto_clone() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Main St".to_string(),
            gender: Gender::Male,
        };

        let cloned_dto = register_dto.clone();
        assert_eq!(register_dto.email, cloned_dto.email);
        assert_eq!(register_dto.password, cloned_dto.password);
        assert_eq!(register_dto.first_name, cloned_dto.first_name);
        assert_eq!(register_dto.last_name, cloned_dto.last_name);
        assert_eq!(register_dto.username, cloned_dto.username);
        assert_eq!(register_dto.birthday, cloned_dto.birthday);
        assert_eq!(register_dto.address, cloned_dto.address);
        assert_eq!(register_dto.gender, cloned_dto.gender);
    }

    #[test]
    fn test_user_login_dto_serialization() {
        let login_dto = UserLoginDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        // Test serialization
        let serialized = serde_json::to_string(&login_dto).unwrap();
        assert!(serialized.contains("test@example.com"));
        assert!(serialized.contains("password123"));

        // Test deserialization
        let deserialized: UserLoginDto = serde_json::from_str(&serialized).unwrap();
        assert_eq!(login_dto.email, deserialized.email);
        assert_eq!(login_dto.password, deserialized.password);
    }

    #[test]
    fn test_user_register_dto_serialization() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Main St".to_string(),
            gender: Gender::Male,
        };

        // Test serialization
        let serialized = serde_json::to_string(&register_dto).unwrap();
        assert!(serialized.contains("test@example.com"));
        assert!(serialized.contains("John"));

        // Test deserialization
        let deserialized: UserRegisterDto = serde_json::from_str(&serialized).unwrap();
        assert_eq!(register_dto.email, deserialized.email);
        assert_eq!(register_dto.first_name, deserialized.first_name);
        assert_eq!(register_dto.last_name, deserialized.last_name);
        assert_eq!(register_dto.username, deserialized.username);
        assert_eq!(register_dto.birthday, deserialized.birthday);
        assert_eq!(register_dto.address, deserialized.address);
        assert_eq!(register_dto.gender, deserialized.gender);
    }

    #[test]
    fn test_user_register_dto_address_too_short() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "".to_string(), // Empty address
            gender: Gender::Male,
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("address"));
    }

    #[test]
    fn test_user_register_dto_address_too_long() {
        let register_dto = UserRegisterDto {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "a".repeat(257), // More than 256 characters
            gender: Gender::Male,
        };

        let validation_result = register_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("address"));
    }

    #[test]
    fn test_user_register_dto_with_different_genders() {
        let male_dto = UserRegisterDto {
            email: "male@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            username: "johndoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1990, 1, 1).unwrap(),
            address: "123 Main St".to_string(),
            gender: Gender::Male,
        };

        let female_dto = UserRegisterDto {
            email: "female@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            username: "janedoe".to_string(),
            birthday: NaiveDate::from_ymd_opt(1992, 5, 15).unwrap(),
            address: "456 Oak Ave".to_string(),
            gender: Gender::Female,
        };

        let other_dto = UserRegisterDto {
            email: "other@example.com".to_string(),
            password: "password123".to_string(),
            first_name: "Alex".to_string(),
            last_name: "Smith".to_string(),
            username: "alexsmith".to_string(),
            birthday: NaiveDate::from_ymd_opt(1985, 12, 25).unwrap(),
            address: "789 Pine Rd".to_string(),
            gender: Gender::Other,
        };

        assert!(male_dto.validate().is_ok());
        assert!(female_dto.validate().is_ok());
        assert!(other_dto.validate().is_ok());
        
        assert_eq!(male_dto.gender, Gender::Male);
        assert_eq!(female_dto.gender, Gender::Female);
        assert_eq!(other_dto.gender, Gender::Other);
    }

    #[test]
    fn test_profile_update_dto_valid_complete() {
        let update_dto = ProfileUpdateDto {
            first_name: Some("Jane".to_string()),
            last_name: Some("Smith".to_string()),
            birthday: Some(NaiveDate::from_ymd_opt(1995, 3, 20).unwrap()),
            address: Some("789 Pine St".to_string()),
            gender: Some(Gender::Female),
        };

        assert!(update_dto.validate().is_ok());
        assert_eq!(update_dto.first_name, Some("Jane".to_string()));
        assert_eq!(update_dto.last_name, Some("Smith".to_string()));
        assert_eq!(update_dto.birthday, Some(NaiveDate::from_ymd_opt(1995, 3, 20).unwrap()));
        assert_eq!(update_dto.address, Some("789 Pine St".to_string()));
        assert_eq!(update_dto.gender, Some(Gender::Female));
    }

    #[test]
    fn test_profile_update_dto_valid_partial() {
        let update_dto = ProfileUpdateDto {
            first_name: Some("UpdatedName".to_string()),
            last_name: None,
            birthday: None,
            address: Some("Updated Address".to_string()),
            gender: None,
        };

        assert!(update_dto.validate().is_ok());
        assert_eq!(update_dto.first_name, Some("UpdatedName".to_string()));
        assert_eq!(update_dto.last_name, None);
        assert_eq!(update_dto.birthday, None);
        assert_eq!(update_dto.address, Some("Updated Address".to_string()));
        assert_eq!(update_dto.gender, None);
    }

    #[test]
    fn test_profile_update_dto_valid_empty() {
        let update_dto = ProfileUpdateDto {
            first_name: None,
            last_name: None,
            birthday: None,
            address: None,
            gender: None,
        };

        assert!(update_dto.validate().is_ok());
        assert!(update_dto.first_name.is_none());
        assert!(update_dto.last_name.is_none());
        assert!(update_dto.birthday.is_none());
        assert!(update_dto.address.is_none());
        assert!(update_dto.gender.is_none());
    }

    #[test]
    fn test_profile_update_dto_address_too_long() {
        let update_dto = ProfileUpdateDto {
            first_name: None,
            last_name: None,
            birthday: None,
            address: Some("a".repeat(257)), // More than 256 characters
            gender: None,
        };

        let validation_result = update_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("address"));
    }

    #[test]
    fn test_profile_update_dto_address_empty_string() {
        let update_dto = ProfileUpdateDto {
            first_name: None,
            last_name: None,
            birthday: None,
            address: Some("".to_string()), // Empty string
            gender: None,
        };

        let validation_result = update_dto.validate();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.field_errors().contains_key("address"));
    }

    #[test]
    fn test_profile_update_dto_clone() {
        let update_dto = ProfileUpdateDto {
            first_name: Some("Jane".to_string()),
            last_name: Some("Smith".to_string()),
            birthday: Some(NaiveDate::from_ymd_opt(1995, 3, 20).unwrap()),
            address: Some("789 Pine St".to_string()),
            gender: Some(Gender::Female),
        };

        let cloned_dto = update_dto.clone();
        assert_eq!(update_dto.first_name, cloned_dto.first_name);
        assert_eq!(update_dto.last_name, cloned_dto.last_name);
        assert_eq!(update_dto.birthday, cloned_dto.birthday);
        assert_eq!(update_dto.address, cloned_dto.address);
        assert_eq!(update_dto.gender, cloned_dto.gender);
    }

    #[test]
    fn test_profile_update_dto_serialization() {
        let update_dto = ProfileUpdateDto {
            first_name: Some("Jane".to_string()),
            last_name: Some("Smith".to_string()),
            birthday: Some(NaiveDate::from_ymd_opt(1995, 3, 20).unwrap()),
            address: Some("789 Pine St".to_string()),
            gender: Some(Gender::Female),
        };

        // Test serialization
        let serialized = serde_json::to_string(&update_dto).unwrap();
        assert!(serialized.contains("Jane"));
        assert!(serialized.contains("Smith"));
        assert!(serialized.contains("789 Pine St"));

        // Test deserialization
        let deserialized: ProfileUpdateDto = serde_json::from_str(&serialized).unwrap();
        assert_eq!(update_dto.first_name, deserialized.first_name);
        assert_eq!(update_dto.last_name, deserialized.last_name);
        assert_eq!(update_dto.birthday, deserialized.birthday);
        assert_eq!(update_dto.address, deserialized.address);
        assert_eq!(update_dto.gender, deserialized.gender);
    }

    #[test]
    fn test_profile_update_dto_debug_format() {
        let update_dto = ProfileUpdateDto {
            first_name: Some("Jane".to_string()),
            last_name: Some("Smith".to_string()),
            birthday: Some(NaiveDate::from_ymd_opt(1995, 3, 20).unwrap()),
            address: Some("789 Pine St".to_string()),
            gender: Some(Gender::Female),
        };

        let debug_string = format!("{:?}", update_dto);
        assert!(debug_string.contains("Jane"));
        assert!(debug_string.contains("Smith"));
        assert!(debug_string.contains("789 Pine St"));
        assert!(debug_string.contains("Female"));
    }

    #[test]
    fn test_profile_update_dto_with_different_genders() {
        let male_dto = ProfileUpdateDto {
            first_name: Some("John".to_string()),
            last_name: None,
            birthday: None,
            address: None,
            gender: Some(Gender::Male),
        };

        let female_dto = ProfileUpdateDto {
            first_name: Some("Jane".to_string()),
            last_name: None,
            birthday: None,
            address: None,
            gender: Some(Gender::Female),
        };

        let other_dto = ProfileUpdateDto {
            first_name: Some("Alex".to_string()),
            last_name: None,
            birthday: None,
            address: None,
            gender: Some(Gender::Other),
        };

        assert!(male_dto.validate().is_ok());
        assert!(female_dto.validate().is_ok());
        assert!(other_dto.validate().is_ok());
        
        assert_eq!(male_dto.gender, Some(Gender::Male));
        assert_eq!(female_dto.gender, Some(Gender::Female));
        assert_eq!(other_dto.gender, Some(Gender::Other));
    }
}