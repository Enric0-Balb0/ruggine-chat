use crate::entity::user::{UserStatus, UserType};
use crate::error::{api_error::ApiError, token_error::TokenError, user_error::UserError};
use crate::state::token_state::TokenState;
use axum::extract::State;
use axum::{http, http::Request, middleware::Next, body::Body};
use futures::future::BoxFuture;
use jsonwebtoken::errors::ErrorKind;
use headers::authorization::{Authorization, Bearer};
use headers::Header;
use axum::response::Response;

pub fn auth(
    allowed_user_types: Vec<UserType>,
) -> impl Clone
         + Fn(State<TokenState>, Request<axum::body::Body>, Next) -> BoxFuture<'static, Result<Response, ApiError>>
         + Send
         + Sync
         + 'static {
    move |State(state), req, next| {
        let allowed_user_types = allowed_user_types.clone();
        Box::pin(async move {
            let req = auth_inner(&state, req, allowed_user_types).await?;
            Ok(next.run(req).await)
        })
    }
}


pub async fn auth_inner(
    state: &TokenState,
    mut req: Request<Body>,
    allowed_user_types: Vec<UserType>,
) -> Result<Request<Body>, ApiError> {
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
    if user.user_status != UserStatus::Active {
        return Err(UserError::UserNotActive.into());
    }

    // Check user type if specified
    if !allowed_user_types.contains(&user.user_type) {
        return Err(UserError::InsufficientPermissions.into());
    }

    // Insert the user into the request extensions for downstream handlers
    req.extensions_mut().insert(user);

    Ok(req)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, header},
        response::{IntoResponse, Response},
    };
    use std::sync::Arc;
    use crate::{
        entity::user::{all_user_types, User},
        factory::{token_factory::TokenFactory, user_factory::UserFactory},
        repository::user_repository::MockUserRepositoryTrait,
        service::token_service::MockTokenServiceTrait,
        state::token_state::TokenState,
    };
    use jsonwebtoken::errors::ErrorKind;

    /// FakeNext simula il comportamento di Next nel test
    struct FakeNext;

    impl FakeNext {
        fn run<B: Send + 'static>(&self, req: Request<B>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response<Body>> + Send>> {
            Box::pin(async move {
                assert!(req.extensions().get::<User>().is_some(), "User not injected");
                // Risposta convertita con IntoResponse per ottenere il corretto tipo Body
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
                user.user_status = UserStatus::Active;
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

        let req_with_user = auth_inner(&state, req, all_user_types()).await.expect("Auth failed");

        let next = FakeNext;
        let response = next.run(req_with_user).await;

        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
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

        let result = auth_inner(&state, req, all_user_types()).await;
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

        let result = auth_inner(&state, req, all_user_types()).await;
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

        let result = auth_inner(&state, req, all_user_types()).await;
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

        let result = auth_inner(&state, req, all_user_types()).await;
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

        let result = auth_inner(&state, req, all_user_types()).await;
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
                user.user_status = UserStatus::Deleted; // User is not active
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

        let result = auth_inner(&state, req, all_user_types()).await;
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

        let result = auth_inner(&state, req, all_user_types()).await;
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

        let result = auth_inner(&state, req, all_user_types()).await;
        assert!(result.is_err());
        
        if let Err(ApiError::TokenError(TokenError::InvalidToken(_))) = result {
            // Expected error
        } else {
            panic!("Expected TokenError::InvalidToken");
        }
    }

    #[tokio::test]
    async fn test_auth_with_allowed_user_type_success() {
        let mut mock_token_service = MockTokenServiceTrait::new();
        mock_token_service
            .expect_retrieve_token_claims()
            .returning(|_| Ok(TokenFactory::fake_token_data()));

        let mut mock_user_repo = MockUserRepositoryTrait::new();
        mock_user_repo.expect_find_by_email().returning(|_| {
            Box::pin(async {
                let mut user = UserFactory::fake_user();
                user.user_status = UserStatus::Active;
                user.user_type = UserType::Admin;
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

        let allowed_types = vec![UserType::Admin, UserType::Developer];
        let result = auth_inner(&state, req, allowed_types).await;
        assert!(result.is_ok(), "Auth should succeed with allowed user type");
    }

    #[tokio::test]
    async fn test_auth_with_disallowed_user_type_fails() {
        let mut mock_token_service = MockTokenServiceTrait::new();
        mock_token_service
            .expect_retrieve_token_claims()
            .returning(|_| Ok(TokenFactory::fake_token_data()));

        let mut mock_user_repo = MockUserRepositoryTrait::new();
        mock_user_repo.expect_find_by_email().returning(|_| {
            Box::pin(async {
                let mut user = UserFactory::fake_user();
                user.user_status = UserStatus::Active;
                user.user_type = UserType::EndUser;
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

        let allowed_types = vec![UserType::Admin];
        let result = auth_inner(&state, req, allowed_types).await;
        assert!(result.is_err(), "Auth should fail with disallowed user type");
        
        if let Err(ApiError::UserError(UserError::InsufficientPermissions)) = result {
            // Expected error
        } else {
            panic!("Expected UserError::InsufficientPermissions");
        }
    }

}
