//! Сценарий получения собственного публичного профиля (`GET /accounts/me`).

use std::sync::Arc;

use uuid::Uuid;

use crate::application::ApplicationError;
use crate::application::dto::AccountView;
use crate::application::ports::AccountRepository;
use crate::domain::UserId;

/// Возвращает публичное представление аккаунта. Никогда не отдаёт хеш пароля.
pub struct GetAccountUseCase {
    accounts: Arc<dyn AccountRepository>,
}

impl GetAccountUseCase {
    pub fn new(accounts: Arc<dyn AccountRepository>) -> Self {
        Self { accounts }
    }

    #[tracing::instrument(skip_all, fields(user_id = %user_id))]
    pub async fn execute(&self, user_id: Uuid) -> Result<AccountView, ApplicationError> {
        let account = self
            .accounts
            .find_by_id(UserId::from_uuid(user_id))
            .await?
            .ok_or(ApplicationError::AccountNotFound)?;

        Ok(AccountView {
            user_id: account.id().as_uuid(),
            nickname: account.nickname().as_str().to_owned(),
            created_at: account.created_at(),
        })
    }
}
