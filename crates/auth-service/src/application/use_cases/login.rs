//! Сценарий входа (login).

use std::sync::Arc;

use time::OffsetDateTime;

use crate::application::ApplicationError;
use crate::application::dto::{LoginCommand, TokenPair};
use crate::application::event::UserEvent;
use crate::application::ports::{
    AccountRepository, EventPublisher, PasswordHasher, RefreshTokenStore, TokenService,
};
use crate::application::use_cases::issue_token_pair;
use crate::domain::{Nickname, PasswordHash};

/// Валидный по формату, но заведомо не совпадающий argon2id-хеш.
///
/// Используется, когда аккаунт не найден: мы всё равно выполняем `verify`,
/// чтобы время ответа не зависело от существования пользователя
/// (защита от timing/username-enumeration атак).
const DUMMY_ARGON2_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHRzb21lc2FsdA$8p5f0m0oq2r4s6u8w0y2A4C6E8G0I2K4M6O8Q0S2U4";

/// Вход: поиск по никнейму → проверка пароля → выпуск токенов →
/// событие `UserLoggedIn`.
pub struct LoginUseCase {
    accounts: Arc<dyn AccountRepository>,
    hasher: Arc<dyn PasswordHasher>,
    tokens: Arc<dyn TokenService>,
    refresh_store: Arc<dyn RefreshTokenStore>,
    events: Arc<dyn EventPublisher>,
}

impl LoginUseCase {
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
    pub async fn execute(&self, command: LoginCommand) -> Result<TokenPair, ApplicationError> {
        // Невалидный формат никнейма ⇒ такого аккаунта быть не может.
        // Отдаём общий InvalidCredentials, не раскрывая деталей.
        let nickname = match Nickname::parse(&command.nickname) {
            Ok(n) => n,
            Err(_) => return Err(ApplicationError::InvalidCredentials),
        };

        let account = match self.accounts.find_by_nickname(&nickname).await? {
            Some(account) => account,
            None => {
                // Constant-time-подобная заглушка: тратим то же время на verify.
                let dummy = PasswordHash::from_hash(DUMMY_ARGON2_HASH)
                    .expect("dummy hash is a valid non-empty string");
                let _ = self.hasher.verify(&command.password, &dummy).await;
                return Err(ApplicationError::InvalidCredentials);
            }
        };

        let password_ok = self
            .hasher
            .verify(&command.password, account.password_hash())
            .await?;
        if !password_ok {
            return Err(ApplicationError::InvalidCredentials);
        }

        let pair = issue_token_pair(
            self.tokens.as_ref(),
            self.refresh_store.as_ref(),
            account.id(),
        )
        .await?;

        let event = UserEvent::LoggedIn {
            user_id: account.id().as_uuid(),
            occurred_at: OffsetDateTime::now_utc(),
        };
        if let Err(err) = self.events.publish(&event).await {
            tracing::error!(error = %err, user_id = %account.id(), "failed to publish UserLoggedIn");
        }

        tracing::info!(user_id = %account.id(), "user logged in");
        Ok(pair)
    }
}
