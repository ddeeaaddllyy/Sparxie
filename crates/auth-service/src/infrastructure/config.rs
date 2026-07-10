//! Конфигурация auth-сервиса, собираемая из переменных окружения.

use std::time::Duration;

use shared::config::{self, ConfigError};

/// Полная конфигурация процесса `nedovolen-auth`.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub server: ServerConfig,
    pub database_url: String,
    pub redis_url: String,
    pub kafka_brokers: String,
    pub kafka_user_events_topic: String,
    pub jwt: JwtConfig,
    pub request_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Параметры JWT. Подпись — EdDSA (Ed25519): приватный ключ у auth-сервиса,
/// публичный раздаётся сервисам-клиентам для оффлайн-валидации.
#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub private_key_path: String,
    pub public_key_path: String,
    pub issuer: String,
    pub access_ttl: Duration,
    pub refresh_ttl: Duration,
}

impl AuthConfig {
    /// Загружает конфигурацию из окружения. Все значения с разумными
    /// дефолтами, кроме внешних адресов и путей к ключам.
    pub fn from_env() -> Result<Self, ConfigError> {
        let server = ServerConfig {
            host: config::optional("SERVER_HOST", "0.0.0.0"),
            port: config::optional_parsed("SERVER_PORT", 8080_u16)?,
        };

        let jwt = JwtConfig {
            private_key_path: config::required("JWT_PRIVATE_KEY_PATH")?,
            public_key_path: config::required("JWT_PUBLIC_KEY_PATH")?,
            issuer: config::optional("JWT_ISSUER", "nedovolen"),
            access_ttl: Duration::from_secs(config::optional_parsed(
                "ACCESS_TOKEN_TTL_SECS",
                900_u64,
            )?),
            refresh_ttl: Duration::from_secs(config::optional_parsed(
                "REFRESH_TOKEN_TTL_SECS",
                1_209_600_u64, // 14 дней
            )?),
        };

        Ok(Self {
            server,
            database_url: config::required("DATABASE_URL")?,
            redis_url: config::required("REDIS_URL")?,
            kafka_brokers: config::required("KAFKA_BROKERS")?,
            kafka_user_events_topic: config::optional(
                "KAFKA_USER_EVENTS_TOPIC",
                "nedovolen.user.events",
            ),
            jwt,
            request_timeout: Duration::from_secs(config::optional_parsed(
                "REQUEST_TIMEOUT_SECS",
                15_u64,
            )?),
        })
    }

    /// Адрес для привязки TCP-листенера.
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}
