//! Порты прикладного слоя.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::RequiemProfile;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("profile not found")]
    NotFound,
    #[error("email already in use")]
    EmailConflict,
    #[error(transparent)]
    Backend(#[from] anyhow::Error),
}

/// Хранилище профилей RequiemProject.
#[async_trait]
pub trait ProfileRepository: Send + Sync {
    /// Идемпотентно создаёт «пустой» профиль (реакция на `UserRegistered`).
    async fn upsert_shell(&self, user_id: Uuid) -> Result<(), RepositoryError>;

    async fn find(&self, user_id: Uuid) -> Result<Option<RequiemProfile>, RepositoryError>;

    /// Сохраняет изменённые поля профиля (email/display_name).
    async fn save(&self, profile: &RequiemProfile) -> Result<(), RepositoryError>;

    /// Идемпотентно удаляет профиль (реакция на `UserDeleted`).
    async fn delete(&self, user_id: Uuid) -> Result<(), RepositoryError>;
}
