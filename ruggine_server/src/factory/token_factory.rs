use std::sync::atomic::{AtomicU64, Ordering};
use jsonwebtoken::{TokenData, Header};
use crate::dto::token_dto::TokenClaimsDto;
use crate::factory::user_factory::UserFactory;

pub struct TokenFactory;

// Global counter to ensure unique JWT secrets across all tests
static JWT_COUNTER: AtomicU64 = AtomicU64::new(1);

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

    /// Helper function to generate unique JWT secret for each test
    pub fn get_unique_jwt_secret(test_name: &str) -> String {
        let counter = JWT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
        format!("test_secret_{}_{}_{}_{}", test_name, counter, timestamp, std::process::id())
    }

}