use crate::config::parameter;
use crate::dto::token_dto::{TokenClaimsDto, TokenReadDto};
use crate::entity::user::User;
use crate::error::token_error::TokenError;
use chrono;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use mockall::automock;

#[derive(Clone)]
pub struct TokenService {
    secret: String,
}

#[automock]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::token_dto::{TokenClaimsDto, TokenReadDto};
    use crate::entity::user::User;
    use crate::error::token_error::TokenError;
    use crate::factory::user_factory::UserFactory;
    use chrono::Utc;
    use jsonwebtoken::{Algorithm, Header, Validation};
    use mockall::predicate;
    use std::sync::Arc;
    use super::MockTokenServiceTrait;

    fn sample_user() -> User {
        UserFactory::fake_user()
    }

    #[test]
    fn test_generate_token_success() {
        let user = sample_user();
        std::env::set_var("JWT_SECRET", "mysecret");
        let service = TokenService::new();

        let token_result = service.generate_token(user.clone());
        assert!(token_result.is_ok());

        let token_data = token_result.unwrap();
        assert!(!token_data.token.is_empty());
        assert!(token_data.iat <= Utc::now().timestamp());
        assert!(token_data.exp > token_data.iat);
    }

    #[test]
    fn test_retrieve_token_claims_success() {
        std::env::set_var("JWT_SECRET", "mysecret");
        let service = TokenService::new();

        let user = sample_user();
        let token_dto = service.generate_token(user.clone()).unwrap();

        let result = service.retrieve_token_claims(&token_dto.token);
        assert!(result.is_ok());

        let claims = result.unwrap().claims;
        assert_eq!(claims.email, user.email);
        assert_eq!(claims.sub, user.id);
    }

    #[test]
    fn test_retrieve_token_claims_invalid_token() {
        std::env::set_var("JWT_SECRET", "mysecret");
        let service = TokenService::new();

        let invalid_token = "invalid.token.value";

        let result = service.retrieve_token_claims(invalid_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_token_service_trait() {
        let mut mock = MockTokenServiceTrait::new();

        let fake_token = "mock_token".to_string();
        let user = sample_user();

        mock.expect_generate_token()
            .with(predicate::eq(user.clone()))
            .returning(move |_| Ok(TokenReadDto {
                token: fake_token.clone(),
                iat: 1234567890,
                exp: 1234569990,
            }));

        let result = mock.generate_token(user.clone());
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.token, "mock_token");
    }
}
