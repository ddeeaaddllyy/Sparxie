//! Маппинг прикладных ошибок в HTTP-ответы.
//!
//! `ApplicationError` (крейт-локальная) оборачивается в локальный [`AppError`],
//! чтобы обойти orphan-rule и реализовать `IntoResponse`. Наружу отдаётся
//! единый формат [`shared::errors::ApiError`].

use axum::response::{IntoResponse, Response};
use shared::errors::ApiError;

use crate::application::ApplicationError;

/// Транспортная обёртка прикладной ошибки.
pub struct AppError(pub ApplicationError);

impl From<ApplicationError> for AppError {
    fn from(err: ApplicationError) -> Self {
        Self(err)
    }
}

/// Переводит прикладную ошибку в HTTP-ошибку со стабильным кодом.
fn to_api_error(err: ApplicationError) -> ApiError {
    match err {
        ApplicationError::Validation(e) => ApiError::bad_request("VALIDATION_ERROR", e.to_string()),
        ApplicationError::NicknameTaken => {
            ApiError::conflict("NICKNAME_TAKEN", "nickname is already taken")
        }
        ApplicationError::InvalidCredentials => {
            ApiError::unauthorized("INVALID_CREDENTIALS", "invalid credentials")
        }
        ApplicationError::AccountNotFound => {
            ApiError::not_found("ACCOUNT_NOT_FOUND", "account not found")
        }
        ApplicationError::InvalidRefreshToken => {
            ApiError::unauthorized("INVALID_REFRESH_TOKEN", "invalid or expired refresh token")
        }
        ApplicationError::Internal(e) => {
            // Детали внутренней ошибки — только в лог, не наружу.
            tracing::error!(error = %e, "internal application error");
            ApiError::internal()
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        to_api_error(self.0).into_response()
    }
}
