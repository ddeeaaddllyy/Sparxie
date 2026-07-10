//! Обработчики эндпоинтов авторизации.
//!
//! Тонкие: парсинг DTO → команда → вызов use case → маппинг результата.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::application::dto::{LoginCommand, LogoutCommand, RefreshCommand, RegisterCommand};
use crate::presentation::http::AppState;
use crate::presentation::http::dto::auth::{
    LoginRequest, LogoutRequest, RefreshRequest, RegisterRequest, TokenResponse,
};
use crate::presentation::http::error::AppError;
use crate::presentation::http::extract::{AuthenticatedUser, ValidatedJson};

/// `POST /api/v1/auth/register`
pub async fn register(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    let pair = state
        .register
        .execute(RegisterCommand {
            nickname: body.nickname,
            password: body.password,
        })
        .await?;

    Ok((StatusCode::CREATED, Json(TokenResponse::from(pair))))
}

/// `POST /api/v1/auth/login`
pub async fn login(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let pair = state
        .login
        .execute(LoginCommand {
            nickname: body.nickname,
            password: body.password,
        })
        .await?;

    Ok(Json(TokenResponse::from(pair)))
}

/// `POST /api/v1/auth/refresh`
pub async fn refresh(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    let pair = state
        .refresh
        .execute(RefreshCommand {
            refresh_token: body.refresh_token,
        })
        .await?;

    Ok(Json(TokenResponse::from(pair)))
}

/// `POST /api/v1/auth/logout` — требует валидный access-токен.
pub async fn logout(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedJson(body): ValidatedJson<LogoutRequest>,
) -> Result<impl IntoResponse, AppError> {
    state
        .logout
        .execute(LogoutCommand {
            access_jti: user.jti,
            access_expires_at: user.access_expires_at,
            refresh_token: body.refresh_token,
        })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
