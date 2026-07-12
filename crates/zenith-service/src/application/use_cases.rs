//! Сценарии Zenith.

use std::sync::Arc;

use uuid::Uuid;

use crate::application::ApplicationError;
use crate::application::dto::{
    AddFoodCommand, AddWorkoutCommand, FoodView, ProfileView, UpdateProfileCommand, WorkoutView,
};
use crate::application::ports::{EntryRepository, ProfileRepository};
use crate::domain::{FoodEntry, WorkoutEntry};

// ─── Реакции на события шины ──────────────────────────────────────────────────

pub struct ApplyUserRegistered {
    profiles: Arc<dyn ProfileRepository>,
}

impl ApplyUserRegistered {
    pub fn new(profiles: Arc<dyn ProfileRepository>) -> Self {
        Self { profiles }
    }

    #[tracing::instrument(skip_all, fields(user_id = %user_id))]
    pub async fn execute(&self, user_id: Uuid) -> Result<(), ApplicationError> {
        self.profiles.upsert_shell(user_id).await?;
        tracing::info!("zenith profile provisioned");
        Ok(())
    }
}

pub struct ApplyUserDeleted {
    profiles: Arc<dyn ProfileRepository>,
}

impl ApplyUserDeleted {
    pub fn new(profiles: Arc<dyn ProfileRepository>) -> Self {
        Self { profiles }
    }

    #[tracing::instrument(skip_all, fields(user_id = %user_id))]
    pub async fn execute(&self, user_id: Uuid) -> Result<(), ApplicationError> {
        self.profiles.delete(user_id).await?;
        tracing::info!("zenith profile removed");
        Ok(())
    }
}

// ─── Профиль ──────────────────────────────────────────────────────────────────

pub struct GetProfile {
    profiles: Arc<dyn ProfileRepository>,
}

impl GetProfile {
    pub fn new(profiles: Arc<dyn ProfileRepository>) -> Self {
        Self { profiles }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<ProfileView, ApplicationError> {
        let profile = self
            .profiles
            .find(user_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        Ok(ProfileView::from(&profile))
    }
}

pub struct UpdateProfile {
    profiles: Arc<dyn ProfileRepository>,
}

impl UpdateProfile {
    pub fn new(profiles: Arc<dyn ProfileRepository>) -> Self {
        Self { profiles }
    }

    #[tracing::instrument(skip_all, fields(user_id = %command.user_id))]
    pub async fn execute(
        &self,
        command: UpdateProfileCommand,
    ) -> Result<ProfileView, ApplicationError> {
        let mut profile = self
            .profiles
            .find(command.user_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        profile.apply_update(command.height, command.weight, command.age)?;
        self.profiles.save(&profile).await?;

        tracing::info!("zenith profile updated");
        Ok(ProfileView::from(&profile))
    }
}

// ─── Еда ──────────────────────────────────────────────────────────────────────

pub struct AddFood {
    entries: Arc<dyn EntryRepository>,
}

impl AddFood {
    pub fn new(entries: Arc<dyn EntryRepository>) -> Self {
        Self { entries }
    }

    #[tracing::instrument(skip_all, fields(user_id = %command.user_id))]
    pub async fn execute(&self, command: AddFoodCommand) -> Result<FoodView, ApplicationError> {
        let entry = FoodEntry::create(
            command.user_id,
            command.name,
            command.calories,
            command.eaten_at,
        )?;
        self.entries.add_food(&entry).await?;
        Ok(FoodView::from(&entry))
    }
}

pub struct ListFood {
    entries: Arc<dyn EntryRepository>,
}

impl ListFood {
    pub fn new(entries: Arc<dyn EntryRepository>) -> Self {
        Self { entries }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<Vec<FoodView>, ApplicationError> {
        let items = self.entries.list_food(user_id).await?;
        Ok(items.iter().map(FoodView::from).collect())
    }
}

// ─── Тренировки ───────────────────────────────────────────────────────────────

pub struct AddWorkout {
    entries: Arc<dyn EntryRepository>,
}

impl AddWorkout {
    pub fn new(entries: Arc<dyn EntryRepository>) -> Self {
        Self { entries }
    }

    #[tracing::instrument(skip_all, fields(user_id = %command.user_id))]
    pub async fn execute(
        &self,
        command: AddWorkoutCommand,
    ) -> Result<WorkoutView, ApplicationError> {
        let entry = WorkoutEntry::create(
            command.user_id,
            command.kind,
            command.duration_min,
            command.performed_at,
        )?;
        self.entries.add_workout(&entry).await?;
        Ok(WorkoutView::from(&entry))
    }
}

pub struct ListWorkout {
    entries: Arc<dyn EntryRepository>,
}

impl ListWorkout {
    pub fn new(entries: Arc<dyn EntryRepository>) -> Self {
        Self { entries }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<Vec<WorkoutView>, ApplicationError> {
        let items = self.entries.list_workout(user_id).await?;
        Ok(items.iter().map(WorkoutView::from).collect())
    }
}
