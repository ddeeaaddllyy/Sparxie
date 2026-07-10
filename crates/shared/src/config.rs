//! Хелперы для типобезопасного чтения конфигурации из переменных окружения.
//!
//! Конкретную структуру конфигурации каждый сервис описывает у себя (например,
//! `auth-service` → `infrastructure::config::AuthConfig`), но парсинг отдельных
//! значений и обработку ошибок централизуем здесь, чтобы не дублировать код и
//! получать единообразные сообщения об ошибках.

use std::env;
use std::str::FromStr;

/// Ошибка загрузки конфигурации из окружения.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Обязательная переменная окружения не задана.
    #[error("missing required environment variable `{0}`")]
    Missing(String),

    /// Значение переменной невозможно распарсить в целевой тип.
    #[error("invalid value for environment variable `{key}`: {message}")]
    Invalid { key: String, message: String },
}

/// Загружает `.env` (если файл присутствует). Отсутствие файла не является
/// ошибкой — в production переменные приходят из окружения оркестратора.
pub fn load_dotenv() {
    // `ok()` намеренно: отсутствие .env в production — норма.
    let _ = dotenvy::dotenv();
}

/// Читает обязательную строковую переменную окружения.
pub fn required(key: &str) -> Result<String, ConfigError> {
    env::var(key).map_err(|_| ConfigError::Missing(key.to_owned()))
}

/// Читает строковую переменную окружения или возвращает значение по умолчанию.
pub fn optional(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}

/// Читает обязательную переменную и парсит её в тип `T`.
pub fn required_parsed<T>(key: &str) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let raw = required(key)?;
    raw.parse::<T>().map_err(|err| ConfigError::Invalid {
        key: key.to_owned(),
        message: err.to_string(),
    })
}

/// Читает переменную и парсит её в `T`, либо возвращает `default`.
pub fn optional_parsed<T>(key: &str, default: T) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    match env::var(key) {
        Ok(raw) => raw.parse::<T>().map_err(|err| ConfigError::Invalid {
            key: key.to_owned(),
            message: err.to_string(),
        }),
        Err(_) => Ok(default),
    }
}
