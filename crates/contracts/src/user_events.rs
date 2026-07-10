//! События жизненного цикла пользователя и конверт сообщения.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Текущая версия схемы событий пользователя.
pub const USER_EVENTS_VERSION: u16 = 1;

// ─── Полезные нагрузки ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegistered {
    pub user_id: Uuid,
    pub nickname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoggedIn {
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordChanged {
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeleted {
    pub user_id: Uuid,
}

/// Типизированная полезная нагрузка события.
///
/// Внутренне тегируется полем `event_type`, значение — в `payload`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "payload")]
pub enum UserEventPayload {
    UserRegistered(UserRegistered),
    UserLoggedIn(UserLoggedIn),
    PasswordChanged(PasswordChanged),
    UserDeleted(UserDeleted),
}

impl UserEventPayload {
    /// Идентификатор пользователя — используется как ключ партиционирования
    /// Kafka (сохраняет порядок событий на пользователя).
    pub fn user_id(&self) -> Uuid {
        match self {
            UserEventPayload::UserRegistered(e) => e.user_id,
            UserEventPayload::UserLoggedIn(e) => e.user_id,
            UserEventPayload::PasswordChanged(e) => e.user_id,
            UserEventPayload::UserDeleted(e) => e.user_id,
        }
    }

    /// Человекочитаемое имя типа события.
    pub fn kind(&self) -> &'static str {
        match self {
            UserEventPayload::UserRegistered(_) => "UserRegistered",
            UserEventPayload::UserLoggedIn(_) => "UserLoggedIn",
            UserEventPayload::PasswordChanged(_) => "PasswordChanged",
            UserEventPayload::UserDeleted(_) => "UserDeleted",
        }
    }
}

// ─── Конверт ──────────────────────────────────────────────────────────────────

/// Самоописываемый конверт события.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Уникальный идентификатор события (для идемпотентности консюмеров).
    pub event_id: Uuid,
    /// Версия схемы.
    pub version: u16,
    /// Время возникновения события (UTC).
    #[serde(with = "time::serde::rfc3339")]
    pub occurred_at: OffsetDateTime,
    /// Полезная нагрузка (`event_type` + `payload`).
    #[serde(flatten)]
    pub event: UserEventPayload,
}

impl EventEnvelope {
    /// Оборачивает полезную нагрузку в конверт с новым `event_id` и текущей версией.
    pub fn new(event: UserEventPayload, occurred_at: OffsetDateTime) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            version: USER_EVENTS_VERSION,
            occurred_at,
            event,
        }
    }

    /// Ключ партиционирования сообщения.
    pub fn partition_key(&self) -> Uuid {
        self.event.user_id()
    }
}
