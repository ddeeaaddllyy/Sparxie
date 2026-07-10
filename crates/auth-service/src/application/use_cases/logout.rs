//! Сценарий выхода (logout).

use std::sync::Arc;

use crate::application::ApplicationError;
use crate::application::dto::LogoutCommand;
use crate::application::ports::{AccessTokenBlacklist, RefreshTokenStore, TokenService};
use crate::application::use_cases::remaining_ttl;
use crate::domain::UserId;

/// Logout: помещаем текущий access в blacklist (до истечения его TTL) и
/// отзываем refresh. Операция идемпотентна.
pub struct LogoutUseCase {
    tokens: Arc<dyn TokenService>,
    refresh_store: Arc<dyn RefreshTokenStore>,
    blacklist: Arc<dyn AccessTokenBlacklist>,
}

impl LogoutUseCase {
    pub fn new(
        tokens: Arc<dyn TokenService>,
        refresh_store: Arc<dyn RefreshTokenStore>,
        blacklist: Arc<dyn AccessTokenBlacklist>,
    ) -> Self {
        Self {
            tokens,
            refresh_store,
            blacklist,
        }
    }

    #[tracing::instrument(skip_all, fields(access_jti = %command.access_jti))]
    pub async fn execute(&self, command: LogoutCommand) -> Result<(), ApplicationError> {
        // Отзываем access мгновенно через blacklist.
        self.blacklist
            .revoke(command.access_jti, remaining_ttl(command.access_expires_at))
            .await?;

        // Отзываем refresh, если он корректен. Некорректный/просроченный refresh
        // при logout не считаем ошибкой — access уже отозван.
        if let Ok(claims) = self.tokens.verify_refresh(&command.refresh_token) {
            let user_id = UserId::from_uuid(claims.user_id);
            self.refresh_store.revoke(user_id, claims.jti).await?;
        }

        tracing::info!("user logged out");
        Ok(())
    }
}
