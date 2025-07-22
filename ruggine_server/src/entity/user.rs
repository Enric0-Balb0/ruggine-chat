use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow, Default, PartialEq)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub user_name: String,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: i8,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub user_name: String,
    pub email: String,
    pub password: String,
    pub is_active: i8,
}