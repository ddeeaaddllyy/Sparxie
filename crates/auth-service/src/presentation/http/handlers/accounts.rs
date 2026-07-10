//! Обработчики эндпоинтов управления аккаунтом (все — защищённые).

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;

use crate::application::dto::{ChangePasswordCommand, DeleteAccountCommand};
use crate::presentation::http::AppState;
use crate::presentation::http::dto::account::{
    AccountResponse, ChangePasswordRequest, DeleteAccountRequest,
};
use crate::presentation::http::error::AppError;
use crate::presentation::http::extract::{AuthenticatedUser, ValidatedJson};

/// `GET /api/v1/accounts/me`
pub async fn me(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, AppError> {
    let view = state.get_account.execute(user.user_id).await?;
    Ok(Json(AccountResponse::from(view)))
}

/// `PATCH /api/v1/accounts/me/password`
pub async fn change_password(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedJson(body): ValidatedJson<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    state
        .change_password
        .execute(ChangePasswordCommand {
            user_id: user.user_id,
            old_password: body.old_password,
            new_password: body.new_password,
        })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /api/v1/accounts/me`
pub async fn delete_me(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedJson(body): ValidatedJson<DeleteAccountRequest>,
) -> Result<impl IntoResponse, AppError> {
    state
        .delete_account
        .execute(DeleteAccountCommand {
            user_id: user.user_id,
            password: body.password,
        })
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
