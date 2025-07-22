use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TokenReadDto {
    pub(crate) token: String,
    pub(crate) iat: i64,
    pub(crate) exp: i64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TokenClaimsDto {
    pub(crate) sub: i32,
    pub(crate) email: String,
    pub(crate) iat: i64,
    pub(crate) exp: i64,
}