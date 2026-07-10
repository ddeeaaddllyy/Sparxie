//! JWT-middleware: проверяет access-токен и авторизует запрос.
//!
//! Применяется только к защищённым маршрутам. Валидирует подпись EdDSA
//! (локально, по публичному ключу), проверяет blacklist и кладёт
//! [`AuthenticatedUser`] в расширения запроса.

use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use shared::errors::ApiError;

use crate::presentation::http::AppState;
use crate::presentation::http::extract::AuthenticatedUser;

/// Извлекает `Bearer`-токен из заголовка `Authorization`.
fn extract_bearer(req: &Request) -> Option<String> {
    req.headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|token| token.trim().to_owned())
        .filter(|token| !token.is_empty())
}

pub async fn jwt_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = extract_bearer(&req)
        .ok_or_else(|| ApiError::unauthorized("MISSING_TOKEN", "missing bearer token"))?;

    let claims = state.token_service.verify_access(&token).map_err(|_| {
        ApiError::unauthorized("INVALID_ACCESS_TOKEN", "invalid or expired access token")
    })?;

    // Проверка мгновенного отзыва (logout).
    let revoked = state.blacklist.is_revoked(claims.jti).await.map_err(|e| {
        tracing::error!(error = %e, "blacklist check failed");
        ApiError::internal()
    })?;
    if revoked {
        return Err(ApiError::unauthorized("TOKEN_REVOKED", "token has been revoked"));
    }

    req.extensions_mut().insert(AuthenticatedUser {
        user_id: claims.user_id,
        jti: claims.jti,
        access_expires_at: claims.expires_at,
    });

    Ok(next.run(req).await)
}
