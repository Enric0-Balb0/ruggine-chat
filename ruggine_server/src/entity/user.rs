use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use utoipa::ToSchema;

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow, Default, PartialEq)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_status: UserStatus,
    pub user_type: UserType,
    pub birthday: NaiveDate,
    pub is_online: bool,
    pub address: String,
    pub gender: Gender,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub user_status: UserStatus,
    pub user_type: UserType,
    pub birthday: NaiveDate,
    pub address: String,
    pub gender: Gender,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UpdateUser {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub birthday: Option<NaiveDate>,
    pub address: Option<String>,
    pub gender: Option<Gender>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema, Display)]
#[sqlx(type_name = "user_type")]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UserType {
    EndUser,
    Developer,
    Admin,
}

pub fn all_user_types() -> Vec<UserType> {
    vec![UserType::EndUser, UserType::Developer, UserType::Admin]
}

impl Default for UserType {
    fn default() -> Self {
        UserType::EndUser
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema, Display)]
#[sqlx(type_name = "user_status")]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum UserStatus {
    Pending,
    Active,
    Suspended,
    Deleted,
    Banned,
}

impl Default for UserStatus {
    fn default() -> Self {
        UserStatus::Active
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema, Display)]
#[sqlx(type_name = "gender")]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    Other,
}

pub fn all_genders() -> Vec<Gender> {
    vec![Gender::Male, Gender::Female, Gender::Other]
}

impl Default for Gender {
    fn default() -> Self {
        Gender::Other
    }
}

impl UpdateUser {
    pub fn from_dto(dto: crate::dto::user_dto::ProfileUpdateDto) -> Self {
        Self {
            first_name: dto.first_name,
            last_name: dto.last_name,
            birthday: dto.birthday,
            address: dto.address,
            gender: dto.gender,
            updated_at: chrono::Utc::now(),
        }
    }

    pub fn has_updates(&self) -> bool {
        self.first_name.is_some() 
            || self.last_name.is_some() 
            || self.birthday.is_some() 
            || self.address.is_some() 
            || self.gender.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::user_dto::ProfileUpdateDto;
    use crate::factory::user_factory::UserFactory;

    #[test]
    fn test_update_user_from_dto_complete() {
        let dto = ProfileUpdateDto {
            first_name: Some("Jane".to_string()),
            last_name: Some("Smith".to_string()),
            birthday: Some(NaiveDate::from_ymd_opt(1995, 3, 20).unwrap()),
            address: Some("789 Pine St".to_string()),
            gender: Some(Gender::Female),
        };

        let update_user = UpdateUser::from_dto(dto);

        assert_eq!(update_user.first_name, Some("Jane".to_string()));
        assert_eq!(update_user.last_name, Some("Smith".to_string()));
        assert_eq!(update_user.birthday, Some(NaiveDate::from_ymd_opt(1995, 3, 20).unwrap()));
        assert_eq!(update_user.address, Some("789 Pine St".to_string()));
        assert_eq!(update_user.gender, Some(Gender::Female));
        assert!(update_user.has_updates());
    }

    #[test]
    fn test_update_user_from_dto_partial() {
        let dto = ProfileUpdateDto {
            first_name: Some("UpdatedName".to_string()),
            last_name: None,
            birthday: None,
            address: Some("Updated Address".to_string()),
            gender: None,
        };

        let update_user = UpdateUser::from_dto(dto);

        assert_eq!(update_user.first_name, Some("UpdatedName".to_string()));
        assert_eq!(update_user.last_name, None);
        assert_eq!(update_user.birthday, None);
        assert_eq!(update_user.address, Some("Updated Address".to_string()));
        assert_eq!(update_user.gender, None);
        assert!(update_user.has_updates());
    }

    #[test]
    fn test_update_user_from_dto_empty() {
        let dto = ProfileUpdateDto {
            first_name: None,
            last_name: None,
            birthday: None,
            address: None,
            gender: None,
        };

        let update_user = UpdateUser::from_dto(dto);

        assert_eq!(update_user.first_name, None);
        assert_eq!(update_user.last_name, None);
        assert_eq!(update_user.birthday, None);
        assert_eq!(update_user.address, None);
        assert_eq!(update_user.gender, None);
        assert!(!update_user.has_updates());
    }

    #[test]
    fn test_update_user_has_updates_single_field() {
        let dto_with_name = ProfileUpdateDto {
            first_name: Some("John".to_string()),
            last_name: None,
            birthday: None,
            address: None,
            gender: None,
        };

        let update_user = UpdateUser::from_dto(dto_with_name);
        assert!(update_user.has_updates());

        let dto_with_gender = ProfileUpdateDto {
            first_name: None,
            last_name: None,
            birthday: None,
            address: None,
            gender: Some(Gender::Other),
        };

        let update_user_gender = UpdateUser::from_dto(dto_with_gender);
        assert!(update_user_gender.has_updates());
    }

    #[test]
    fn test_update_user_updated_at_is_set() {
        let dto = UserFactory::fake_user_update_dto();
        let before_creation = chrono::Utc::now();
        
        let update_user = UpdateUser::from_dto(dto.clone());
        let after_creation = chrono::Utc::now();

        assert!(update_user.updated_at >= before_creation);
        assert!(update_user.updated_at <= after_creation);
    }

    #[test]
    fn test_update_user_from_factory_dto() {
        let dto = UserFactory::fake_user_update_dto();
        let update_user = UpdateUser::from_dto(dto.clone());

        assert!(update_user.has_updates());
        assert_eq!(update_user.first_name, dto.first_name);
        assert_eq!(update_user.last_name, dto.last_name);
        assert_eq!(update_user.gender, dto.gender);
    }

    #[test]
    fn test_update_user_from_factory_partial_dto() {
        let dto = UserFactory::fake_user_update_dto_partial();
        let update_user = UpdateUser::from_dto(dto.clone());

        assert!(update_user.has_updates());
        assert_eq!(update_user.first_name, dto.first_name);
        assert_eq!(update_user.last_name, dto.last_name);
        assert_eq!(update_user.birthday, dto.birthday);
        assert_eq!(update_user.address, dto.address);
        assert_eq!(update_user.gender, dto.gender);
    }

    #[test]
    fn test_update_user_from_factory_empty_dto() {
        let dto = UserFactory::fake_user_update_dto_empty();
        let update_user = UpdateUser::from_dto(dto);

        assert!(!update_user.has_updates());
        assert!(update_user.first_name.is_none());
        assert!(update_user.last_name.is_none());
        assert!(update_user.birthday.is_none());
        assert!(update_user.address.is_none());
        assert!(update_user.gender.is_none());
    }
}

