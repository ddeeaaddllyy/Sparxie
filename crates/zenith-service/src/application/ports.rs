//! Порты прикладного слоя Zenith.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{FoodEntry, WorkoutEntry, ZenithProfile};

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("profile not found")]
    NotFound,
    #[error(transparent)]
    Backend(#[from] anyhow::Error),
}

/// Хранилище профилей Zenith.
#[async_trait]
pub trait ProfileRepository: Send + Sync {
    /// Идемпотентно создаёт «пустой» профиль (реакция на `UserRegistered`).
    async fn upsert_shell(&self, user_id: Uuid) -> Result<(), RepositoryError>;

    async fn find(&self, user_id: Uuid) -> Result<Option<ZenithProfile>, RepositoryError>;

    async fn save(&self, profile: &ZenithProfile) -> Result<(), RepositoryError>;

    /// Удаляет профиль; записи еды/тренировок удаляются каскадно.
    async fn delete(&self, user_id: Uuid) -> Result<(), RepositoryError>;
}

/// Хранилище записей еды и тренировок.
#[async_trait]
pub trait EntryRepository: Send + Sync {
    /// Возвращает [`RepositoryError::NotFound`], если профиля-владельца нет.
    async fn add_food(&self, entry: &FoodEntry) -> Result<(), RepositoryError>;

    async fn list_food(&self, user_id: Uuid) -> Result<Vec<FoodEntry>, RepositoryError>;

    async fn add_workout(&self, entry: &WorkoutEntry) -> Result<(), RepositoryError>;

    async fn list_workout(&self, user_id: Uuid) -> Result<Vec<WorkoutEntry>, RepositoryError>;
}
