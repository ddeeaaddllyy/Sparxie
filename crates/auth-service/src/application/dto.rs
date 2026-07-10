//! Прикладные DTO: команды (вход use case) и результаты (выход use case).
//!
//! Это НЕ HTTP-DTO. HTTP-контракты (с `serde`/`validator`) живут в presentation
//! и маппятся в эти команды. Так прикладной слой не зависит от транспорта.

use time::OffsetDateTime;
use uuid::Uuid;

// ─── Команды ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct RegisterCommand {
    pub nickname: String,
    pub password: String,
}

#[derive(Debug)]
pub struct LoginCommand {
    pub nickname: String,
    pub password: String,
}

#[derive(Debug)]
pub struct RefreshCommand {
    pub refresh_token: String,
}

/// Данные для logout: identity берётся из проверенного access-токена
/// (JWT middleware), refresh — из тела запроса.
#[derive(Debug)]
pub struct LogoutCommand {
    pub access_jti: Uuid,
    pub access_expires_at: OffsetDateTime,
    pub refresh_token: String,
}

#[derive(Debug)]
pub struct ChangePasswordCommand {
    pub user_id: Uuid,
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug)]
pub struct DeleteAccountCommand {
    pub user_id: Uuid,
    pub password: String,
}

// ─── Результаты ───────────────────────────────────────────────────────────────

/// Пара токенов, выдаваемая при регистрации/входе/обновлении.
#[derive(Debug, Clone)]
pub struct TokenPair {
    pub user_id: Uuid,
    pub access_token: String,
    pub access_expires_at: OffsetDateTime,
    pub refresh_token: String,
    pub refresh_expires_at: OffsetDateTime,
}

/// Публичное представление аккаунта (для `GET /accounts/me`).
/// Никогда не содержит `password_hash`.
#[derive(Debug, Clone)]
pub struct AccountView {
    pub user_id: Uuid,
    pub nickname: String,
    pub created_at: OffsetDateTime,
}
