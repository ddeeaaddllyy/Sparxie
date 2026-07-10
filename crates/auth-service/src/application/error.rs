//! Ошибки прикладного слоя.
//!
//! Инфраструктурные (порты) и доменные ошибки сворачиваются сюда. Маппинг в
//! HTTP-статусы делает слой presentation (этап 4), поэтому здесь нет ничего
//! от Axum.

use crate::application::ports::{
    BlacklistError, PasswordHasherError, PublishError, RefreshStoreError, RepositoryError,
};
use crate::domain::DomainError;

/// Ошибка выполнения сценария (use case).
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    /// Невалидные входные данные (нарушение доменного инварианта).
    #[error("validation error: {0}")]
    Validation(#[from] DomainError),

    /// Никнейм уже занят при регистрации.
    #[error("nickname is already taken")]
    NicknameTaken,

    /// Неверная пара nickname/пароль (общий ответ, без утечки деталей).
    #[error("invalid credentials")]
    InvalidCredentials,

    /// Аккаунт не найден.
    #[error("account not found")]
    AccountNotFound,

    /// Refresh-токен недействителен, истёк или отозван.
    #[error("invalid or expired refresh token")]
    InvalidRefreshToken,

    /// Непредвиденная внутренняя ошибка (инфраструктура). Наружу не раскрывается.
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<RepositoryError> for ApplicationError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => ApplicationError::AccountNotFound,
            RepositoryError::NicknameConflict => ApplicationError::NicknameTaken,
            RepositoryError::Backend(e) => ApplicationError::Internal(e),
        }
    }
}

impl From<PasswordHasherError> for ApplicationError {
    fn from(err: PasswordHasherError) -> Self {
        // Детали хеширования — всегда внутренняя ошибка.
        ApplicationError::Internal(anyhow::Error::new(err))
    }
}

impl From<RefreshStoreError> for ApplicationError {
    fn from(err: RefreshStoreError) -> Self {
        match err {
            RefreshStoreError::Backend(e) => ApplicationError::Internal(e),
        }
    }
}

impl From<BlacklistError> for ApplicationError {
    fn from(err: BlacklistError) -> Self {
        match err {
            BlacklistError::Backend(e) => ApplicationError::Internal(e),
        }
    }
}

impl From<PublishError> for ApplicationError {
    fn from(err: PublishError) -> Self {
        match err {
            PublishError::Backend(e) => ApplicationError::Internal(e),
        }
    }
}
