use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TokenReadDto {
    pub token: String,
    pub iat: i64,
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