use chrono::{DateTime, Utc};
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
    pub user_type: UserType
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub user_status: UserStatus,
    pub user_type: UserType
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