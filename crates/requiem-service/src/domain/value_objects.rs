//! Value-объекты профиля.

use std::fmt;

use super::error::DomainError;

/// Email пользователя (лёгкая валидация формата).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    const MAX_LEN: usize = 254;

    pub fn parse(raw: impl Into<String>) -> Result<Self, DomainError> {
        let raw = raw.into();
        let trimmed = raw.trim();

        if trimmed.len() > Self::MAX_LEN {
            return Err(DomainError::InvalidEmail);
        }

        // Простой инвариант: ровно один '@', непустые локальная и доменная части,
        // в домене есть точка.
        let parts: Vec<&str> = trimmed.split('@').collect();
        let valid = parts.len() == 2
            && !parts[0].is_empty()
            && !parts[1].is_empty()
            && parts[1].contains('.')
            && !trimmed.contains(char::is_whitespace);

        if !valid {
            return Err(DomainError::InvalidEmail);
        }

        Ok(Self(trimmed.to_ascii_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Отображаемое имя.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayName(String);

impl DisplayName {
    const MIN_LEN: usize = 1;
    const MAX_LEN: usize = 64;

    pub fn parse(raw: impl Into<String>) -> Result<Self, DomainError> {
        let raw = raw.into();
        let trimmed = raw.trim();
        let len = trimmed.chars().count();

        if len < Self::MIN_LEN || len > Self::MAX_LEN {
            return Err(DomainError::InvalidDisplayName);
        }

        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DisplayName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
