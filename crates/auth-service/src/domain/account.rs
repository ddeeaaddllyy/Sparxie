//! Сущность `Account` — корень агрегата аккаунта.
//!
//! Общая база `nedovolen` хранит ровно эти поля: `uuid`, `nickname`,
//! `password_hash`, `created_at`, `updated_at`. Никаких email/профилей —
//! они принадлежат сервисам-клиентам.

use time::OffsetDateTime;

use super::value_objects::{Nickname, PasswordHash, UserId};

/// Аккаунт пользователя.
#[derive(Debug, Clone)]
pub struct Account {
    id: UserId,
    nickname: Nickname,
    password_hash: PasswordHash,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl Account {
    /// Создаёт новый аккаунт при регистрации (генерирует `UserId` и метки времени).
    pub fn register(nickname: Nickname, password_hash: PasswordHash) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: UserId::new(),
            nickname,
            password_hash,
            created_at: now,
            updated_at: now,
        }
    }

    /// Восстанавливает сущность из строки БД (репозиторий).
    pub fn from_persistence(
        id: UserId,
        nickname: Nickname,
        password_hash: PasswordHash,
        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
    ) -> Self {
        Self {
            id,
            nickname,
            password_hash,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    pub fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }

    pub fn updated_at(&self) -> OffsetDateTime {
        self.updated_at
    }
}
