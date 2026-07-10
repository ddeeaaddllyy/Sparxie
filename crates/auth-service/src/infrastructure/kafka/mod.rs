//! Kafka-инфраструктура: публикация доменных событий.

pub mod producer;

pub use producer::KafkaEventPublisher;
