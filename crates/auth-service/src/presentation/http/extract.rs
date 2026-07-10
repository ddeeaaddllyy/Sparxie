//! Кастомные экстракторы Axum.
//!
//! * [`ValidatedJson`] — десериализует тело и валидирует его (`validator`).
//! * [`AuthenticatedUser`] — достаёт данные аутентифицированного пользователя,
//!   которые положил в расширения запроса JWT-middleware.

use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use axum::Json;
use serde::de::DeserializeOwned;
use shared::errors::ApiError;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

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

/// Аутентифицированный пользователь (заполняется JWT-middleware).
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub jti: Uuid,
    pub access_expires_at: OffsetDateTime,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| ApiError::unauthorized("UNAUTHORIZED", "authentication required"))
    }
}
