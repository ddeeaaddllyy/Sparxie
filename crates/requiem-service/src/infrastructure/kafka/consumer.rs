//! Kafka-консюмер событий пользователя.
//!
//! Читает топик `nedovolen.user.events`, десериализует [`EventEnvelope`] и
//! диспетчеризует в use cases. Реагирует на `UserRegistered`/`UserDeleted`,
//! остальные события игнорирует. Обработчики идемпотентны (at-least-once).

use std::sync::Arc;

use rdkafka::Message;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};

use contracts::{EventEnvelope, UserEventPayload};

use crate::application::use_cases::{ApplyUserDeleted, ApplyUserRegistered};
use crate::infrastructure::config::KafkaConfig;

pub struct EventConsumer {
    consumer: StreamConsumer,
    topic: String,
    on_registered: Arc<ApplyUserRegistered>,
    on_deleted: Arc<ApplyUserDeleted>,
}

impl EventConsumer {
    pub fn new(
        kafka: &KafkaConfig,
        on_registered: Arc<ApplyUserRegistered>,
        on_deleted: Arc<ApplyUserDeleted>,
    ) -> anyhow::Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &kafka.brokers)
            .set("group.id", &kafka.group_id)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", "earliest")
            .create()?;

        Ok(Self {
            consumer,
            topic: kafka.topic.clone(),
            on_registered,
            on_deleted,
        })
    }

    /// Бесконечный цикл потребления. Запускается в отдельной задаче.
    pub async fn run(self) -> anyhow::Result<()> {
        self.consumer.subscribe(&[self.topic.as_str()])?;
        tracing::info!(topic = %self.topic, "kafka consumer started");

        loop {
            match self.consumer.recv().await {
                Err(e) => tracing::error!(error = %e, "kafka receive error"),
                Ok(message) => {
                    let Some(payload) = message.payload() else {
                        continue;
                    };
                    if let Err(e) = self.handle(payload).await {
                        // Ошибку логируем; offset не откатываем (auto-commit).
                        // Идемпотентность обработчиков защищает от дублей.
                        tracing::error!(error = %e, "failed to handle event");
                    }
                }
            }
        }
    }

    async fn handle(&self, payload: &[u8]) -> anyhow::Result<()> {
        let envelope: EventEnvelope = serde_json::from_slice(payload)?;

        match &envelope.event {
            UserEventPayload::UserRegistered(e) => self.on_registered.execute(e.user_id).await?,
            UserEventPayload::UserDeleted(e) => self.on_deleted.execute(e.user_id).await?,
            other => tracing::debug!(kind = other.kind(), "event ignored by requiem-service"),
        }
        Ok(())
    }
}
