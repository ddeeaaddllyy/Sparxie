//! Сценарий смены пароля.

use std::sync::Arc;

use time::OffsetDateTime;

use crate::application::ApplicationError;
use crate::application::dto::ChangePasswordCommand;
use crate::application::event::UserEvent;
use crate::application::ports::{
    AccountRepository, EventPublisher, PasswordHasher, RefreshTokenStore,
};
use crate::domain::{Password, UserId};

/// Смена пароля: проверка старого → хеширование нового → сохранение →
/// отзыв всех refresh-сессий → событие `PasswordChanged`.
pub struct ChangePasswordUseCase {
    accounts: Arc<dyn AccountRepository>,
    hasher: Arc<dyn PasswordHasher>,
    refresh_store: Arc<dyn RefreshTokenStore>,
    events: Arc<dyn EventPublisher>,
}

impl ChangePasswordUseCase {
    pub fn new(
        accounts: Arc<dyn AccountRepository>,
        hasher: Arc<dyn PasswordHasher>,
        refresh_store: Arc<dyn RefreshTokenStore>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            accounts,
            hasher,
            refresh_store,
            events,
        }
    }

    #[tracing::instrument(skip_all, fields(user_id = %command.user_id))]
    pub async fn execute(&self, command: ChangePasswordCommand) -> Result<(), ApplicationError> {
        let new_password = Password::parse(command.new_password)?;
        let user_id = UserId::from_uuid(command.user_id);

        let account = self
            .accounts
            .find_by_id(user_id)
            .await?
            .ok_or(ApplicationError::AccountNotFound)?;

        let old_ok = self
            .hasher
            .verify(&command.old_password, account.password_hash())
            .await?;
        if !old_ok {
            return Err(ApplicationError::InvalidCredentials);
        }

        let new_hash = self.hasher.hash(new_password.expose()).await?;
        self.accounts.update_password(user_id, &new_hash).await?;

        // Все активные сессии становятся недействительны.
        self.refresh_store.revoke_all(user_id).await?;

        let event = UserEvent::PasswordChanged {
            user_id: user_id.as_uuid(),
            occurred_at: OffsetDateTime::now_utc(),
        };
        if let Err(err) = self.events.publish(&event).await {
            tracing::error!(error = %err, user_id = %user_id, "failed to publish PasswordChanged");
        }

        tracing::info!(user_id = %user_id, "password changed");
        Ok(())
    }
}
