//! DTO эндпоинтов авторизации.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::application::dto::TokenPair;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 32, message = "nickname must be 3-32 characters"))]
    pub nickname: String,
    #[validate(length(min = 8, max = 128, message = "password must be 8-128 characters"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "nickname is required"))]
    pub nickname: String,
    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RefreshRequest {
    #[validate(length(min = 1, message = "refresh_token is required"))]
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LogoutRequest {
    #[validate(length(min = 1, message = "refresh_token is required"))]
    pub refresh_token: String,
}

/// Ответ с парой токенов.
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub user_id: Uuid,
    pub token_type: &'static str,
    pub access_token: String,
    #[serde(with = "time::serde::rfc3339")]
    pub access_expires_at: OffsetDateTime,
    pub refresh_token: String,
    #[serde(with = "time::serde::rfc3339")]
    pub refresh_expires_at: OffsetDateTime,
}

impl From<TokenPair> for TokenResponse {
    fn from(pair: TokenPair) -> Self {
        Self {
            user_id: pair.user_id,
            token_type: "Bearer",
            access_token: pair.access_token,
            access_expires_at: pair.access_expires_at,
            refresh_token: pair.refresh_token,
            refresh_expires_at: pair.refresh_expires_at,
        }
    }
}
