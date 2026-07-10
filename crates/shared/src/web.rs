//! Переиспользуемые HTTP-компоненты для сервисов-клиентов.
//!
//! * [`ValidatedJson`] — JSON-тело с валидацией (`validator`).
//! * [`AuthUser`] + [`jwt_auth`] — аутентификация по access-токену, проверяемому
//!   локально через [`crate::jwt::AccessTokenVerifier`].
//!
//! Сервис реализует [`HasAccessVerifier`] для своего `AppState` — и получает
//! готовый JWT-middleware.

use axum::Json;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, FromRequestParts, Request, State};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use serde::de::DeserializeOwned;
use uuid::Uuid;
use validator::Validate;

use crate::errors::ApiError;
use crate::jwt::AccessTokenVerifier;

/// JSON-тело, прошедшее валидацию.
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection: JsonRejection| {
                ApiError::bad_request("INVALID_BODY", rejection.body_text())
            })?;
        value
            .validate()
            .map_err(|e| ApiError::bad_request("VALIDATION_ERROR", e.to_string()))?;
        Ok(ValidatedJson(value))
    }
}

/// Аутентифицированный пользователь (кладётся [`jwt_auth`] в расширения запроса).
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub jti: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| ApiError::unauthorized("UNAUTHORIZED", "authentication required"))
    }
}

/// Состояние сервиса, предоставляющее верификатор токенов.
pub trait HasAccessVerifier: Clone + Send + Sync + 'static {
    fn access_verifier(&self) -> &AccessTokenVerifier;
}

fn extract_bearer(req: &Request) -> Option<String> {
    req.headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|t| t.trim().to_owned())
        .filter(|t| !t.is_empty())
}

/// Middleware аутентификации: проверяет access-токен и кладёт [`AuthUser`]
/// в расширения запроса. Применяется к защищённым маршрутам.
pub async fn jwt_auth<S: HasAccessVerifier>(
    State(state): State<S>,
    mut req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = extract_bearer(&req)
        .ok_or_else(|| ApiError::unauthorized("MISSING_TOKEN", "missing bearer token"))?;

    let verified = state.access_verifier().verify(&token).map_err(|_| {
        ApiError::unauthorized("INVALID_ACCESS_TOKEN", "invalid or expired access token")
    })?;

    req.extensions_mut().insert(AuthUser {
        user_id: verified.user_id,
        jti: verified.jti,
    });

    Ok(next.run(req).await)
}
