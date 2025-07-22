use std::sync::atomic::AtomicU32;
use jsonwebtoken::{TokenData, Header};
use crate::dto::token_dto::TokenClaimsDto;
use crate::factory::user_factory::UserFactory;

pub struct TokenFactory;

impl TokenFactory{
    pub fn fake_token_data() -> TokenData<TokenClaimsDto> {
        TokenData {
            header: Header::default(),
            claims: TokenClaimsDto {
                sub: 1,
                email: "john.doe@example.com".into(),
                iat: 0,
                exp: 9999999999,
            },
        }
    }

    pub fn fake_unique_token_data(prefix: &str) -> TokenData<TokenClaimsDto> {
        let (email, _, _) = UserFactory::get_unique_user_information(prefix);
        TokenData {
            header: Header::default(),
            claims: TokenClaimsDto {
                sub: 1,
                email,
                iat: 0,
                exp: 9999999999,
            },
        }
    }
}