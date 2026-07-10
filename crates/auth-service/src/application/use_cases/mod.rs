//! Сценарии (use cases) прикладного слоя.
//!
//! Каждый use case — самостоятельная единица бизнес-логики, зависящая только
//! от портов. Один тип — один сценарий (Single Responsibility).

mod change_password;
mod delete_account;
mod get_account;
mod login;
mod logout;
mod refresh;
mod register;

pub use change_password::ChangePasswordUseCase;
pub use delete_account::DeleteAccountUseCase;
pub use get_account::GetAccountUseCase;
pub use login::LoginUseCase;
pub use logout::LogoutUseCase;
pub use refresh::RefreshUseCase;
pub use register::RegisterUseCase;

use std::time::Duration;

use time::OffsetDateTime;

use crate::application::ApplicationError;
use crate::application::dto::TokenPair;
use crate::application::ports::{RefreshTokenStore, TokenError, TokenService};
use crate::domain::UserId;

/// Остаток времени жизни (для TTL в Redis). Никогда не отрицательный.
pub(crate) fn remaining_ttl(expires_at: OffsetDateTime) -> Duration {
    let secs = (expires_at - OffsetDateTime::now_utc()).whole_seconds();
    if secs <= 0 {
        Duration::from_secs(0)
    } else {
        Duration::from_secs(secs as u64)
    }
}

/// Ошибка ВЫПУСКА токена — всегда внутренняя (Invalid/Expired при выпуске
/// невозможны, ключ наш).
pub(crate) fn issue_failed(err: TokenError) -> ApplicationError {
    ApplicationError::Internal(anyhow::anyhow!("token issue failed: {err}"))
}

/// Выпускает пару access+refresh и регистрирует refresh в whitelist.
/// Общий шаг для register / login / refresh.
pub(crate) async fn issue_token_pair(
    tokens: &dyn TokenService,
    refresh_store: &dyn RefreshTokenStore,
    user_id: UserId,
) -> Result<TokenPair, ApplicationError> {
    let access = tokens.issue_access(user_id).map_err(issue_failed)?;
    let refresh = tokens.issue_refresh(user_id).map_err(issue_failed)?;

    refresh_store
        .store(user_id, refresh.jti, remaining_ttl(refresh.expires_at))
        .await?;

    Ok(TokenPair {
        user_id: user_id.as_uuid(),
        access_token: access.token,
        access_expires_at: access.expires_at,
        refresh_token: refresh.token,
        refresh_expires_at: refresh.expires_at,
    })
}
