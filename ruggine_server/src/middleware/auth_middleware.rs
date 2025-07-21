use crate::error::{api_error::ApiError, token_error::TokenError, user_error::UserError};
use crate::repository::user_repository::UserRepositoryTrait;
use crate::service::token_service::TokenServiceTrait;
use crate::state::token_state::TokenState;
use axum::extract::State;
use axum::{http, http::Request, middleware::Next, response::IntoResponse};
use jsonwebtoken::errors::ErrorKind;
use axum::headers::authorization::{Authorization, Bearer};
use axum::headers::Header;

pub async fn auth<B>(
    State(state): State<TokenState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, ApiError> {
    // Extract the Authorization Bearer header
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

    // Continue the middleware chain
    Ok(next.run(req).await)
}
