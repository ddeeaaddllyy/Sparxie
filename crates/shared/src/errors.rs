//! Единый формат HTTP-ошибки для всех сервисов.
//!
//! Тело ответа всегда имеет форму:
//! ```json
//! { "error": { "code": "INVALID_CREDENTIALS", "message": "..." } }
//! ```
//! Прикладные ошибки (`ApplicationError` конкретного сервиса) маппятся в
//! [`ApiError`] на слое presentation.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Машиночитаемое тело ошибки.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

/// Внешний конверт ошибки.
#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: ErrorBody,
}

/// Ошибка, готовая к сериализации в HTTP-ответ.
///
/// `code` — стабильный машиночитаемый идентификатор (для клиентов),
/// `message` — человекочитаемое описание (не должно содержать секретов).
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    pub fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }

    pub fn unauthorized(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, code, message)
    }

    pub fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, code, message)
    }

    pub fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, code, message)
    }

    /// Внутренняя ошибка. Детали намеренно не раскрываются наружу —
    /// их пишет в лог слой presentation.
    pub fn internal() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "internal server error",
        )
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorResponse {
            error: ErrorBody {
                code: self.code.to_owned(),
                message: self.message,
            },
        };
        (self.status, Json(body)).into_response()
    }
}
