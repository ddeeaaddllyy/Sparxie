//! Маппинг прикладных ошибок в HTTP.

use axum::response::{IntoResponse, Response};
use shared::errors::ApiError;

use crate::application::ApplicationError;

pub struct AppError(pub ApplicationError);

impl From<ApplicationError> for AppError {
    fn from(err: ApplicationError) -> Self {
        Self(err)
    }
}

fn to_api_error(err: ApplicationError) -> ApiError {
    match err {
        ApplicationError::Validation(e) => ApiError::bad_request("VALIDATION_ERROR", e.to_string()),
        ApplicationError::NotFound => ApiError::not_found("PROFILE_NOT_FOUND", "profile not found"),
        ApplicationError::Internal(e) => {
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
