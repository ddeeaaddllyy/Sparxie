//! Сценарии RequiemProject.
//!
//! Два вида: реакции на события шины (`apply_*`) и обработчики REST-запросов.

use std::sync::Arc;

use uuid::Uuid;

use crate::application::ApplicationError;
use crate::application::dto::{ProfileView, UpdateProfileCommand};
use crate::application::ports::ProfileRepository;
use crate::domain::{DisplayName, Email};

/// Реакция на `UserRegistered`: создать «пустой» профиль (идемпотентно).
pub struct ApplyUserRegistered {
    repo: Arc<dyn ProfileRepository>,
}

impl ApplyUserRegistered {
    pub fn new(repo: Arc<dyn ProfileRepository>) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip_all, fields(user_id = %user_id))]
    pub async fn execute(&self, user_id: Uuid) -> Result<(), ApplicationError> {
        self.repo.upsert_shell(user_id).await?;
        tracing::info!("requiem profile provisioned");
        Ok(())
    }
}

/// Реакция на `UserDeleted`: удалить профиль (идемпотентно).
pub struct ApplyUserDeleted {
    repo: Arc<dyn ProfileRepository>,
}

impl ApplyUserDeleted {
    pub fn new(repo: Arc<dyn ProfileRepository>) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip_all, fields(user_id = %user_id))]
    pub async fn execute(&self, user_id: Uuid) -> Result<(), ApplicationError> {
        self.repo.delete(user_id).await?;
        tracing::info!("requiem profile removed");
        Ok(())
    }
}

/// `GET /profile/me`.
pub struct GetProfile {
    repo: Arc<dyn ProfileRepository>,
}

impl GetProfile {
    pub fn new(repo: Arc<dyn ProfileRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, user_id: Uuid) -> Result<ProfileView, ApplicationError> {
        let profile = self
            .repo
            .find(user_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        Ok(ProfileView::from(&profile))
    }
}

/// `PUT /profile/me`.
pub struct UpdateProfile {
    repo: Arc<dyn ProfileRepository>,
}

impl UpdateProfile {
    pub fn new(repo: Arc<dyn ProfileRepository>) -> Self {
        Self { repo }
    }

    #[tracing::instrument(skip_all, fields(user_id = %command.user_id))]
    pub async fn execute(
        &self,
        command: UpdateProfileCommand,
    ) -> Result<ProfileView, ApplicationError> {
        // Валидация только тех полей, что переданы.
        let email = command.email.map(Email::parse).transpose()?;
        let display_name = command.display_name.map(DisplayName::parse).transpose()?;

        let mut profile = self
            .repo
            .find(command.user_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;

        profile.apply_update(email, display_name);
        self.repo.save(&profile).await?;

        tracing::info!("requiem profile updated");
        Ok(ProfileView::from(&profile))
    }
}
