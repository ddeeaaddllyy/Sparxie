//! Публикация событий пользователя в Kafka.
//!
//! Прикладное событие [`UserEvent`] маппится в конверт [`EventEnvelope`] из
//! крейта `contracts`, сериализуется в JSON и отправляется с ключом = `user_id`.

use std::time::Duration;

use async_trait::async_trait;
use rdkafka::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};

use contracts::{
    EventEnvelope, PasswordChanged, UserDeleted, UserEventPayload, UserLoggedIn, UserRegistered,
};

use crate::application::event::UserEvent;
use crate::application::ports::{EventPublisher, PublishError};

/// Таймаут доставки одного сообщения продюсером.
const SEND_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct KafkaEventPublisher {
    producer: FutureProducer,
    topic: String,
}

impl KafkaEventPublisher {
    pub fn new(brokers: &str, topic: String) -> anyhow::Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            // Идемпотентный продюсер: без дублей и с гарантией порядка при ретраях.
            .set("enable.idempotence", "true")
            .create()?;

        Ok(Self { producer, topic })
    }
}

/// Преобразует прикладное событие в транспортный конверт.
fn to_envelope(event: &UserEvent) -> EventEnvelope {
    match event {
        UserEvent::Registered {
            user_id,
            nickname,
            occurred_at,
        } => EventEnvelope::new(
            UserEventPayload::UserRegistered(UserRegistered {
                user_id: *user_id,
                nickname: nickname.clone(),
            }),
            *occurred_at,
        ),
        UserEvent::LoggedIn {
            user_id,
            occurred_at,
        } => EventEnvelope::new(
            UserEventPayload::UserLoggedIn(UserLoggedIn { user_id: *user_id }),
            *occurred_at,
        ),
        UserEvent::PasswordChanged {
            user_id,
            occurred_at,
        } => EventEnvelope::new(
            UserEventPayload::PasswordChanged(PasswordChanged { user_id: *user_id }),
            *occurred_at,
        ),
        UserEvent::Deleted {
            user_id,
            occurred_at,
        } => EventEnvelope::new(
            UserEventPayload::UserDeleted(UserDeleted { user_id: *user_id }),
            *occurred_at,
        ),
    }
}

#[async_trait]
impl EventPublisher for KafkaEventPublisher {
    async fn publish(&self, event: &UserEvent) -> Result<(), PublishError> {
        let envelope = to_envelope(event);
        let key = envelope.partition_key().to_string();
        let kind = envelope.event.kind();

        let payload = serde_json::to_vec(&envelope)
            .map_err(|e| PublishError::Backend(anyhow::anyhow!("serialize event: {e}")))?;

        let record = FutureRecord::to(&self.topic)
            .key(key.as_str())
            .payload(payload.as_slice());

        self.producer
            .send(record, SEND_TIMEOUT)
            .await
            .map_err(|(e, _)| {
                PublishError::Backend(anyhow::anyhow!("kafka send failed for {kind}: {e}"))
            })?;

        tracing::debug!(event = kind, key = %key, "event published to kafka");
        Ok(())
    }
}
