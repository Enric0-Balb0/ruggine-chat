use crate::error::{api_error::ApiError, token_error::TokenError, user_error::UserError};
use crate::state::token_state::TokenState;
use axum::extract::State;
use axum::{http, http::Request, middleware::Next, response::IntoResponse};
use jsonwebtoken::errors::ErrorKind;
use axum::headers::authorization::{Authorization, Bearer};
use axum::headers::Header;

/// Middleware principale: chiama la logica di auth_inner e poi next.run
pub async fn auth<B>(
    State(state): State<TokenState>,
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, ApiError> {
    let req = auth_inner(&state, req).await?;
    Ok(next.run(req).await)
}

pub async fn auth_inner<B>(
    state: &TokenState,
    mut req: Request<B>,
) -> Result<Request<B>, ApiError> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .ok_or(TokenError::MissingToken)?;

    // Decode the header into Authorization<Bearer>
    let header = Authorization::<Bearer>::decode(&mut std::iter::once(auth_header))
        .map_err(|_| TokenError::MissingToken)?;

    let token = header.token();

    // Validate the token and retrieve claims
    let token_data = state
        .token_service
        .retrieve_token_claims(token)
        .map_err(|err| match err.kind() {
            ErrorKind::ExpiredSignature => TokenError::TokenExpired,
            _ => TokenError::InvalidToken(token.to_string()),
        })?;

    // Find the user associated with the email from the claims
    let user = state.user_repo.find_by_email(token_data.claims.email).await
        .ok_or(UserError::UserNotFound)?;

    // Check if the user is active
    if user.is_active == 0 {
        return Err(UserError::UserNotActive.into());
    }

    // Insert the user into the request extensions for downstream handlers
    req.extensions_mut().insert(user);

    Ok(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, BoxBody},
        http::{Request, header},
        response::{IntoResponse, Response},
    };
    use std::sync::Arc;
    use crate::{
        entity::user::User,
        factory::user_factory::UserFactory,
        factory::token_factory::TokenFactory,
        repository::user_repository::MockUserRepositoryTrait,
        service::token_service::MockTokenServiceTrait,
        state::token_state::TokenState,
    };
    use jsonwebtoken::errors::ErrorKind;

    /// FakeNext simula il comportamento di Next nel test
    struct FakeNext;

    impl FakeNext {
        fn run<B: Send + 'static>(&self, req: Request<B>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response<BoxBody>> + Send>> {
            Box::pin(async move {
                assert!(req.extensions().get::<User>().is_some(), "User not injected");
                // Risposta convertita con IntoResponse per ottenere il corretto tipo BoxBody
                ("passed").into_response()
            })
        }
    }

    #[tokio::test]
    async fn test_auth_success() {
        let mut mock_token_service = MockTokenServiceTrait::new();
        mock_token_service
            .expect_retrieve_token_claims()
            .returning(|_| Ok(TokenFactory::fake_token_data()));

        let mut mock_user_repo = MockUserRepositoryTrait::new();
        mock_user_repo.expect_find_by_email().returning(|_| {
            Box::pin(async {
                let mut user = UserFactory::fake_user();
                user.is_active = 1;
                Some(user)
            })
        });

        let state = TokenState {
            token_service: Arc::new(mock_token_service),
            user_repo: Arc::new(mock_user_repo),
        };

        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, "Bearer validtoken")
            .body(Body::empty())
            .unwrap();

        let req_with_user = auth_inner(&state, req).await.expect("Auth failed");

        let next = FakeNext;
        let response = next.run(req_with_user).await;

        let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let body_str = std::str::from_utf8(&bytes).unwrap();

        assert_eq!(body_str, "passed");
    }

    #[tokio::test]
    async fn test_auth_missing_authorization_header() {
        let mock_token_service = MockTokenServiceTrait::new();
        let mock_user_repo = MockUserRepositoryTrait::new();

        let state = TokenState {
            token_service: Arc::new(mock_token_service),
            user_repo: Arc::new(mock_user_repo),
        };

        let req = Request::builder()
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let result = auth_inner(&state, req).await;
        assert!(result.is_err());
        
        if let Err(ApiError::TokenError(TokenError::MissingToken)) = result {
            // Expected error
        } else {
            panic!("Expected TokenError::MissingToken");
        }
    }

    #[tokio::test]
    async fn test_auth_invalid_authorization_header() {
        let mock_token_service = MockTokenServiceTrait::new();
        let mock_user_repo = MockUserRepositoryTrait::new();

        let state = TokenState {
            token_service: Arc::new(mock_token_service),
            user_repo: Arc::new(mock_user_repo),
        };

        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, "InvalidHeader")
            .body(Body::empty())
            .unwrap();

        let result = auth_inner(&state, req).await;
        assert!(result.is_err());
        
        if let Err(ApiError::TokenError(TokenError::MissingToken)) = result {
            // Expected error
        } else {
            panic!("Expected TokenError::MissingToken");
        }
    }

    #[tokio::test]
    async fn test_auth_expired_token() {
        let mut mock_token_service = MockTokenServiceTrait::new();
        mock_token_service
            .expect_retrieve_token_claims()
            .returning(|_| {
                let error = jsonwebtoken::errors::Error::from(ErrorKind::ExpiredSignature);
                Err(error)
            });

        let mock_user_repo = MockUserRepositoryTrait::new();

        let state = TokenState {
            token_service: Arc::new(mock_token_service),
            user_repo: Arc::new(mock_user_repo),
        };

        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, "Bearer expiredtoken")
            .body(Body::empty())
            .unwrap();

        let result = auth_inner(&state, req).await;
        assert!(result.is_err());
        
        if let Err(ApiError::TokenError(TokenError::TokenExpired)) = result {
            // Expected error
        } else {
            panic!("Expected TokenError::TokenExpired");
        }
    }

    #[tokio::test]
    async fn test_auth_invalid_token() {
        let mut mock_token_service = MockTokenServiceTrait::new();
        mock_token_service
            .expect_retrieve_token_claims()
            .returning(|_| {
                let error = jsonwebtoken::errors::Error::from(ErrorKind::InvalidToken);
                Err(error)
            });

        let mock_user_repo = MockUserRepositoryTrait::new();

        let state = TokenState {
            token_service: Arc::new(mock_token_service),
            user_repo: Arc::new(mock_user_repo),
        };

        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, "Bearer invalidtoken")
            .body(Body::empty())
            .unwrap();

        let result = auth_inner(&state, req).await;
        assert!(result.is_err());
        
        if let Err(ApiError::TokenError(TokenError::InvalidToken(_))) = result {
            // Expected error
        } else {
            panic!("Expected TokenError::InvalidToken");
        }
    }

    #[tokio::test]
    async fn test_auth_user_not_found() {
        let mut mock_token_service = MockTokenServiceTrait::new();
        mock_token_service
            .expect_retrieve_token_claims()
            .returning(|_| Ok(TokenFactory::fake_unique_token_data("notfound")));

        let mut mock_user_repo = MockUserRepositoryTrait::new();
        mock_user_repo.expect_find_by_email().returning(|_| {
            Box::pin(async { None })
        });

        let state = TokenState {
            token_service: Arc::new(mock_token_service),
            user_repo: Arc::new(mock_user_repo),
        };

        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, "Bearer validtoken")
            .body(Body::empty())
            .unwrap();

        let result = auth_inner(&state, req).await;
        assert!(result.is_err());
        
        if let Err(ApiError::UserError(UserError::UserNotFound)) = result {
            // Expected error
        } else {
            panic!("Expected UserError::UserNotFound");
        }
    }

    #[tokio::test]
    async fn test_auth_user_not_active() {
        let mut mock_token_service = MockTokenServiceTrait::new();
        mock_token_service
            .expect_retrieve_token_claims()
            .returning(|_| Ok(TokenFactory::fake_unique_token_data("inactive")));

        let mut mock_user_repo = MockUserRepositoryTrait::new();
        mock_user_repo.expect_find_by_email().returning(|_| {
            Box::pin(async {
                let mut user = UserFactory::fake_user();
                user.is_active = 0; // User is not active
                Some(user)
            })
        });

        let state = TokenState {
            token_service: Arc::new(mock_token_service),
            user_repo: Arc::new(mock_user_repo),
        };

        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, "Bearer validtoken")
            .body(Body::empty())
            .unwrap();

        let result = auth_inner(&state, req).await;
        assert!(result.is_err());
        
        if let Err(ApiError::UserError(UserError::UserNotActive)) = result {
            // Expected error
        } else {
            panic!("Expected UserError::UserNotActive");
        }
    }

    #[tokio::test]
    async fn test_auth_bearer_token_without_bearer_prefix() {
        let mock_token_service = MockTokenServiceTrait::new();
        let mock_user_repo = MockUserRepositoryTrait::new();

        let state = TokenState {
            token_service: Arc::new(mock_token_service),
            user_repo: Arc::new(mock_user_repo),
        };

        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, "sometoken")
            .body(Body::empty())
            .unwrap();

        let result = auth_inner(&state, req).await;
        assert!(result.is_err());
        
        if let Err(ApiError::TokenError(TokenError::MissingToken)) = result {
            // Expected error
        } else {
            panic!("Expected TokenError::MissingToken");
        }
    }

    #[tokio::test]
    async fn test_auth_empty_bearer_token() {
        let mut mock_token_service = MockTokenServiceTrait::new();
        mock_token_service
            .expect_retrieve_token_claims()
            .returning(|_| {
                let error = jsonwebtoken::errors::Error::from(ErrorKind::InvalidToken);
                Err(error)
            });

        let mock_user_repo = MockUserRepositoryTrait::new();

        let state = TokenState {
            token_service: Arc::new(mock_token_service),
            user_repo: Arc::new(mock_user_repo),
        };

        let req = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, "Bearer ")
            .body(Body::empty())
            .unwrap();

        let result = auth_inner(&state, req).await;
        assert!(result.is_err());
        
        if let Err(ApiError::TokenError(TokenError::InvalidToken(_))) = result {
            // Expected error
        } else {
            panic!("Expected TokenError::InvalidToken");
        }
    }

}
