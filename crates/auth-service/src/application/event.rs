//! Доменные события, публикуемые в шину (Kafka).
//!
//! Здесь описана прикладная форма события. Конкретная сериализация и схема
//! топика (крейт `contracts`) — забота инфраструктуры на этапе 3.

use time::OffsetDateTime;
use uuid::Uuid;

/// Событие жизненного цикла пользователя.
///
/// Ключ партиционирования при публикации — `user_id`, что сохраняет порядок
/// событий в рамках одного пользователя.
#[derive(Debug, Clone)]
pub enum UserEvent {
    Registered {
        user_id: Uuid,
        nickname: String,
        occurred_at: OffsetDateTime,
    },
    LoggedIn {
        user_id: Uuid,
        occurred_at: OffsetDateTime,
    },
    PasswordChanged {
        user_id: Uuid,
        occurred_at: OffsetDateTime,
    },
    Deleted {
        user_id: Uuid,
        occurred_at: OffsetDateTime,
    },
}
