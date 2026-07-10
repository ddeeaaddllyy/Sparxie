//! Value-объекты аккаунта.
//!
//! Каждый тип гарантирует свой инвариант в конструкторе (`parse`/`new`), после
//! чего невалидное состояние в принципе непредставимо в остальной программе
//! («parse, don't validate»).

use std::fmt;

use uuid::Uuid;

use super::error::DomainError;

/// Уникальный идентификатор пользователя во всей экосистеме `nedovolen`.
///
/// Именно он служит внешним ключом во всех сервисах-клиентах.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(Uuid);

impl UserId {
    /// Генерирует новый идентификатор (при регистрации).
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Восстанавливает идентификатор из хранилища / токена.
    pub fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Никнейм — уникальное человекочитаемое имя аккаунта.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Nickname(String);

impl Nickname {
    const MIN_LEN: usize = 3;
    const MAX_LEN: usize = 32;

    /// Проверяет инвариант и создаёт валидный никнейм.
    pub fn parse(raw: impl Into<String>) -> Result<Self, DomainError> {
        let raw = raw.into();
        let len = raw.chars().count();

        if len < Self::MIN_LEN || len > Self::MAX_LEN {
            return Err(DomainError::InvalidNickname);
        }

        let valid_charset = raw
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');

        if !valid_charset {
            return Err(DomainError::InvalidNickname);
        }

        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Nickname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Пароль в открытом виде — **транзиентная** величина.
///
/// Существует лишь на время обработки запроса (хеширование / проверка) и
/// никогда не сохраняется. `Debug` намеренно скрывает содержимое, чтобы пароль
/// не утёк в логи.
#[derive(Clone)]
pub struct Password(String);

impl Password {
    const MIN_LEN: usize = 8;
    const MAX_LEN: usize = 128;

    pub fn parse(raw: impl Into<String>) -> Result<Self, DomainError> {
        let raw = raw.into();
        let len = raw.chars().count();

        if len < Self::MIN_LEN || len > Self::MAX_LEN {
            return Err(DomainError::WeakPassword);
        }

        Ok(Self(raw))
    }

    /// Возвращает открытый текст для передачи в хешер. Использовать только
    /// внутри инфраструктуры хеширования.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Password(***redacted***)")
    }
}

/// Хеш пароля (Argon2id). Хранится в БД; в открытый вид не разворачивается.
#[derive(Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    /// Оборачивает уже посчитанный хешером хеш.
    pub fn from_hash(raw: impl Into<String>) -> Result<Self, DomainError> {
        let raw = raw.into();
        if raw.trim().is_empty() {
            return Err(DomainError::InvalidPasswordHash);
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PasswordHash(***redacted***)")
    }
}
