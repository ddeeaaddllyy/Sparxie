//! Сценарий регистрации нового аккаунта.

use std::sync::Arc;

use time::OffsetDateTime;

use crate::application::ApplicationError;
use crate::application::dto::{RegisterCommand, TokenPair};
use crate::application::event::UserEvent;
use crate::application::ports::{
    AccountRepository, EventPublisher, PasswordHasher, RefreshTokenStore, TokenService,
};
use crate::application::use_cases::issue_token_pair;
use crate::domain::{Account, Nickname, Password};

/// Регистрация: проверка уникальности → хеширование → сохранение → выпуск
/// токенов → событие `UserRegistered`.
pub struct RegisterUseCase {
    accounts: Arc<dyn AccountRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenService>,
    refresh_store: Arc<dyn RefreshTokenStore>,
    events: Arc<dyn EventPublisher>,
}

impl RegisterUseCase {
    pub fn new(
        accounts: Arc<dyn AccountRepository>,
        hasher: Arc<dyn PasswordHasher>,
        tokens: Arc<dyn TokenService>,
        refresh_store: Arc<dyn RefreshTokenStore>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            accounts,
            hasher,
            tokens,
            refresh_store,
            events,
        }
    }

    #[tracing::instrument(skip_all, fields(nickname = %command.nickname))]
    pub async fn execute(&self, command: RegisterCommand) -> Result<TokenPair, ApplicationError> {
        let nickname = Nickname::parse(command.nickname)?;
        let password = Password::parse(command.password)?;

        if self.accounts.exists_by_nickname(&nickname).await? {
            return Err(ApplicationError::NicknameTaken);
        }

        let password_hash = self.hasher.hash(password.expose()).await?;
        let account = Account::register(nickname, password_hash);

        // `insert` также вернёт NicknameTaken при гонке (unique violation).
        self.accounts.insert(&account).await?;

        let pair = issue_token_pair(
            self.tokens.as_ref(),
            self.refresh_store.as_ref(),
            account.id(),
        )
        .await?;

        // Публикация события — best-effort. На этапе 3 будет заменена
        // транзакционным outbox для гарантированной доставки.
        let event = UserEvent::Registered {
            user_id: account.id().as_uuid(),
            nickname: account.nickname().as_str().to_owned(),
            occurred_at: OffsetDateTime::now_utc(),
        };
        if let Err(err) = self.events.publish(&event).await {
            tracing::error!(error = %err, user_id = %account.id(), "failed to publish UserRegistered");
        }

        tracing::info!(user_id = %account.id(), "account registered");
        Ok(pair)
    }
}
