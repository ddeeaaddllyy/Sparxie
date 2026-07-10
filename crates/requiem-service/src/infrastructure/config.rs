//! Конфигурация requiem-service.

use std::time::Duration;

use shared::config::{self, ConfigError};

#[derive(Debug, Clone)]
pub struct RequiemConfig {
    pub server: ServerConfig,
    pub database_url: String,
    pub kafka: KafkaConfig,
    pub jwt: JwtConfig,
    pub request_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct KafkaConfig {
    pub brokers: String,
    pub topic: String,
    pub group_id: String,
}

/// Клиент проверяет токены только публичным ключом — приватный ему недоступен.
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub public_key_path: String,
    pub issuer: String,
}

impl RequiemConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            server: ServerConfig {
                host: config::optional("SERVER_HOST", "0.0.0.0"),
                port: config::optional_parsed("SERVER_PORT", 8081_u16)?,
            },
            database_url: config::required("DATABASE_URL")?,
            kafka: KafkaConfig {
                brokers: config::required("KAFKA_BROKERS")?,
                topic: config::optional("KAFKA_USER_EVENTS_TOPIC", "nedovolen.user.events"),
                group_id: config::optional("KAFKA_GROUP_ID", "requiem-service"),
            },
            jwt: JwtConfig {
                public_key_path: config::required("JWT_PUBLIC_KEY_PATH")?,
                issuer: config::optional("JWT_ISSUER", "nedovolen"),
            },
            request_timeout: Duration::from_secs(config::optional_parsed(
                "REQUEST_TIMEOUT_SECS",
                15_u64,
            )?),
        })
    }

    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}
