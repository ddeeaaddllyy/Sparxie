//! Сценарий обновления пары токенов (refresh с ротацией).

use std::sync::Arc;

use crate::application::ApplicationError;
use crate::application::dto::{RefreshCommand, TokenPair};
use crate::application::ports::{
    RefreshTokenStore, TokenError, TokenService,
};
use crate::application::use_cases::issue_token_pair;
use crate::domain::UserId;

/// Refresh: проверка подписи → проверка whitelist → отзыв старого →
/// выпуск новой пары (rotation).
///
/// Ротация делает refresh-токены одноразовыми: повторное использование уже
/// обменянного токена не пройдёт проверку whitelist.
pub struct RefreshUseCase {
    tokens: Arc<dyn TokenService>,
    refresh_store: Arc<dyn RefreshTokenStore>,
}

impl RefreshUseCase {
    pub fn new(tokens: Arc<dyn TokenService>, refresh_store: Arc<dyn RefreshTokenStore>) -> Self {
        Self {
            tokens,
            refresh_store,
        }
    }

    #[tracing::instrument(skip_all)]
    pub async fn execute(&self, command: RefreshCommand) -> Result<TokenPair, ApplicationError> {
        let claims = self
            .tokens
            .verify_refresh(&command.refresh_token)
            .map_err(|err| match err {
                TokenError::Invalid | TokenError::Expired => {
                    ApplicationError::InvalidRefreshToken
                }
                TokenError::Issue(e) => ApplicationError::Internal(e),
            })?;

        let user_id = UserId::from_uuid(claims.user_id);

        if !self.refresh_store.is_active(user_id, claims.jti).await? {
            return Err(ApplicationError::InvalidRefreshToken);
        }

        // Отзываем предъявленный refresh — он одноразовый.
        self.refresh_store.revoke(user_id, claims.jti).await?;

        let pair =
            issue_token_pair(self.tokens.as_ref(), self.refresh_store.as_ref(), user_id).await?;

        tracing::info!(user_id = %user_id, "tokens refreshed");
        Ok(pair)
    }
}
