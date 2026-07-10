//! DTO эндпоинтов управления аккаунтом.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::application::dto::AccountView;

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "old_password is required"))]
    pub old_password: String,
    #[validate(length(min = 8, max = 128, message = "new password must be 8-128 characters"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct DeleteAccountRequest {
    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}

/// Публичное представление аккаунта. Без `password_hash`.
#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub user_id: Uuid,
    pub nickname: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<AccountView> for AccountResponse {
    fn from(view: AccountView) -> Self {
        Self {
            user_id: view.user_id,
            nickname: view.nickname,
            created_at: view.created_at,
        }
    }
}
