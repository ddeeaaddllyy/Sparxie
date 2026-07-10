//! Сценарий удаления аккаунта.

use std::sync::Arc;

use time::OffsetDateTime;

use crate::application::ApplicationError;
use crate::application::dto::DeleteAccountCommand;
use crate::application::event::UserEvent;
use crate::application::ports::{
    AccountRepository, EventPublisher, PasswordHasher, RefreshTokenStore,
};
use crate::domain::UserId;

/// Удаление: подтверждение паролем → удаление аккаунта → отзыв всех сессий →
/// событие `UserDeleted` (по нему сервисы-клиенты чистят свои проекции).
pub struct DeleteAccountUseCase {
    accounts: Arc<dyn AccountRepository>,
    hasher: Arc<dyn PasswordHasher>,
    refresh_store: Arc<dyn RefreshTokenStore>,
    events: Arc<dyn EventPublisher>,
}

impl DeleteAccountUseCase {
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
    pub async fn execute(&self, command: DeleteAccountCommand) -> Result<(), ApplicationError> {
        let user_id = UserId::from_uuid(command.user_id);

        let account = self
            .accounts
            .find_by_id(user_id)
            .await?
            .ok_or(ApplicationError::AccountNotFound)?;

        let password_ok = self
            .hasher
            .verify(&command.password, account.password_hash())
            .await?;
        if !password_ok {
            return Err(ApplicationError::InvalidCredentials);
        }

        self.accounts.delete(user_id).await?;
        self.refresh_store.revoke_all(user_id).await?;

        let event = UserEvent::Deleted {
            user_id: user_id.as_uuid(),
            occurred_at: OffsetDateTime::now_utc(),
        };
        if let Err(err) = self.events.publish(&event).await {
            tracing::error!(error = %err, user_id = %user_id, "failed to publish UserDeleted");
        }

        tracing::info!(user_id = %user_id, "account deleted");
        Ok(())
    }
}
