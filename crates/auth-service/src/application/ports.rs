//! Порты прикладного слоя — абстракции над инфраструктурой.
//!
//! Use cases зависят только от этих traits; конкретные реализации
//! (Postgres/Redis/Kafka/Argon2/Ed25519) внедряются в `main` через `AppState`.
//! Это и есть инверсия зависимостей Clean Architecture.

use std::time::Duration;

use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::application::event::UserEvent;
use crate::domain::{Account, Nickname, PasswordHash, UserId};

// ─── Ошибки портов ────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("account not found")]
    NotFound,
    #[error("nickname already exists")]
    NicknameConflict,
    #[error(transparent)]
    Backend(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PasswordHasherError {
    #[error("failed to hash password: {0}")]
    Hash(#[source] anyhow::Error),
    #[error("failed to verify password: {0}")]
    Verify(#[source] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("failed to issue token: {0}")]
    Issue(#[source] anyhow::Error),
    #[error("token is invalid")]
    Invalid,
    #[error("token is expired")]
    Expired,
}

#[derive(Debug, thiserror::Error)]
pub enum RefreshStoreError {
    #[error(transparent)]
    Backend(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum BlacklistError {
    #[error(transparent)]
    Backend(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    Backend(#[from] anyhow::Error),
}

// ─── Значения токенов ─────────────────────────────────────────────────────────

/// Только что выпущенный токен.
#[derive(Debug, Clone)]
pub struct IssuedToken {
    pub token: String,
    pub jti: Uuid,
    pub expires_at: OffsetDateTime,
}

/// Полезная нагрузка проверенного access-токена.
#[derive(Debug, Clone)]
pub struct AccessClaims {
    pub user_id: Uuid,
    pub jti: Uuid,
    pub expires_at: OffsetDateTime,
}

/// Полезная нагрузка проверенного refresh-токена.
#[derive(Debug, Clone)]
pub struct RefreshClaims {
    pub user_id: Uuid,
    pub jti: Uuid,
}

// ─── Порты ────────────────────────────────────────────────────────────────────

/// Хранилище аккаунтов (реализация — PostgreSQL/SQLx).
#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn find_by_id(&self, id: UserId) -> Result<Option<Account>, RepositoryError>;

    async fn find_by_nickname(
        &self,
        nickname: &Nickname,
    ) -> Result<Option<Account>, RepositoryError>;

    async fn exists_by_nickname(&self, nickname: &Nickname) -> Result<bool, RepositoryError>;

    /// Вставляет новый аккаунт. Возвращает [`RepositoryError::NicknameConflict`]
    /// при нарушении уникальности никнейма.
    async fn insert(&self, account: &Account) -> Result<(), RepositoryError>;

    async fn update_password(
        &self,
        id: UserId,
        password_hash: &PasswordHash,
    ) -> Result<(), RepositoryError>;

    async fn delete(&self, id: UserId) -> Result<(), RepositoryError>;
}

/// Хеширование паролей (реализация — Argon2id).
#[async_trait]
pub trait PasswordHasher: Send + Sync {
    async fn hash(&self, plaintext: &str) -> Result<PasswordHash, PasswordHasherError>;

    /// Проверяет пароль против хеша. Должна выполняться за константное время
    /// относительно результата (защита от timing-атак).
    async fn verify(
        &self,
        plaintext: &str,
        hash: &PasswordHash,
    ) -> Result<bool, PasswordHasherError>;
}

/// Выпуск и проверка JWT (реализация — Ed25519/EdDSA).
///
/// Синхронный: операции с ключом дёшевы и не блокируют исполнителя надолго.
pub trait TokenService: Send + Sync {
    fn issue_access(&self, user_id: UserId) -> Result<IssuedToken, TokenError>;

    fn issue_refresh(&self, user_id: UserId) -> Result<IssuedToken, TokenError>;

    fn verify_access(&self, token: &str) -> Result<AccessClaims, TokenError>;

    fn verify_refresh(&self, token: &str) -> Result<RefreshClaims, TokenError>;
}

/// Реестр активных refresh-токенов (whitelist в Redis).
///
/// Токен считается действительным, только если присутствует здесь — это даёт
/// возможность отзыва (logout, смена пароля).
#[async_trait]
pub trait RefreshTokenStore: Send + Sync {
    async fn store(
        &self,
        user_id: UserId,
        jti: Uuid,
        ttl: Duration,
    ) -> Result<(), RefreshStoreError>;

    async fn is_active(&self, user_id: UserId, jti: Uuid) -> Result<bool, RefreshStoreError>;

    async fn revoke(&self, user_id: UserId, jti: Uuid) -> Result<(), RefreshStoreError>;

    /// Отзывает все refresh-токены пользователя (смена пароля / удаление).
    async fn revoke_all(&self, user_id: UserId) -> Result<(), RefreshStoreError>;
}

/// Blacklist access-токенов (Redis) — для мгновенного отзыва при logout.
#[async_trait]
pub trait AccessTokenBlacklist: Send + Sync {
    /// Помещает `jti` в blacklist с TTL, равным остатку жизни токена.
    async fn revoke(&self, jti: Uuid, ttl: Duration) -> Result<(), BlacklistError>;

    async fn is_revoked(&self, jti: Uuid) -> Result<bool, BlacklistError>;
}

/// Публикация доменных событий в шину (Kafka).
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &UserEvent) -> Result<(), PublishError>;
}
