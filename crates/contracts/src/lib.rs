//! `contracts` — единый источник правды по схемам событий шины (Kafka).
//!
//! Продюсер (`auth-service`) и будущие консюмеры (`requiem-service`,
//! `zenith-service`) сериализуют/десериализуют события через эти типы, что
//! исключает рассинхронизацию схемы.
//!
//! Формат сообщения — самоописываемый JSON-конверт:
//! ```json
//! {
//!   "event_id":   "…uuid…",
//!   "version":    1,
//!   "occurred_at":"2026-07-09T12:00:00Z",
//!   "event_type": "UserRegistered",
//!   "payload":    { "user_id": "…", "nickname": "…" }
//! }
//! ```

pub mod user_events;

pub use user_events::{
    EventEnvelope, PasswordChanged, UserDeleted, UserEventPayload, UserLoggedIn, UserRegistered,
};

/// Топик по умолчанию для событий жизненного цикла пользователя.
/// Фактическое имя топика конфигурируется через `KAFKA_USER_EVENTS_TOPIC`.
pub const USER_EVENTS_TOPIC: &str = "nedovolen.user.events";
