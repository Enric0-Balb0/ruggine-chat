use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Serialize, Deserialize, Debug, ToSchema)]
#[schema(example = json!({
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "iat": 1640995200,
    "exp": 1641081600
}))]
pub struct TokenReadDto {
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    #[schema(example = 1640995200)]
    pub iat: i64,
    #[schema(example = 1641081600)]
    pub exp: i64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TokenClaimsDto {
    pub(crate) sub: i32,
    pub(crate) email: String,
    pub(crate) iat: i64,
    pub(crate) exp: i64,
}

impl TokenClaimsDto {
    pub fn sub(&self) -> i32 {
        self.sub
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn iat(&self) -> i64 {
        self.iat
    }

    pub fn exp(&self) -> i64 {
        self.exp
    }
}