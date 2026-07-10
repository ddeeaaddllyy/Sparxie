//! Прикладной слой (use cases).
//!
//! Оркестрирует доменные объекты и инфраструктуру через **порты** (traits).
//! Не знает про конкретные Postgres/Redis/Kafka — только про абстракции.
//! Зависимости направлены внутрь: `application` → `domain`.

pub mod dto;
pub mod error;
pub mod event;
pub mod ports;
pub mod use_cases;

pub use error::ApplicationError;
