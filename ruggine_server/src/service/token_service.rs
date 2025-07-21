use crate::config::parameter;
use crate::dto::token_dto::{TokenClaimsDto, TokenReadDto};
use crate::entity::user::User;
use crate::error::token_error::TokenError;
use chrono;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};

#[derive(Clone)]
pub struct TokenService {
    secret: String,
}

pub trait TokenServiceTrait {
    fn retrieve_token_claims(
        &self,
        token: &str,
    ) -> jsonwebtoken::errors::Result<TokenData<TokenClaimsDto>>;

    fn generate_token(&self, user: User) -> Result<TokenReadDto, TokenError>;
}

impl TokenService {
    pub fn new() -> Self {
        Self {
            secret: parameter::get("JWT_SECRET"),
        }
    }

    pub const TOKEN_EXPIRATION: i64 = 30;
}

impl TokenServiceTrait for TokenService {
    fn retrieve_token_claims(
        &self,
        token: &str,
    ) -> jsonwebtoken::errors::Result<TokenData<TokenClaimsDto>> {
        decode::<TokenClaimsDto>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default(),
        )
    }

    fn generate_token(&self, user: User) -> Result<TokenReadDto, TokenError> {
        let iat = chrono::Utc::now().timestamp();
        let exp = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::minutes(Self::TOKEN_EXPIRATION))
            .unwrap()
            .timestamp();

        let claims = TokenClaimsDto {
            sub: user.id,
            email: user.email,
            iat,
            exp,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .map_err(|e| TokenError::TokenCreationError(e.to_string()))?;

        Ok(TokenReadDto { token, iat, exp })
    }
}
