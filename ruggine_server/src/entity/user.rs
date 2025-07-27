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
    pub current_action: CurrentAction,
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
#[sqlx(type_name = "current_action")]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CurrentAction {
    Waiting,
    Writing,
}

impl Default for CurrentAction {
    fn default() -> Self {
        CurrentAction::Waiting
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

impl Default for Gender {
    fn default() -> Self {
        Gender::Other
    }
}

