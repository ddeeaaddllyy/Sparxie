//! Обработчики профиля (защищённые JWT).

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use shared::web::{AuthUser, ValidatedJson};

use crate::application::dto::UpdateProfileCommand;
use crate::presentation::http::AppState;
use crate::presentation::http::dto::{ProfileResponse, UpdateProfileRequest};
use crate::presentation::http::error::AppError;

/// `GET /api/v1/zenith/profile/me`
pub async fn get_me(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let view = state.get_profile.execute(user.user_id).await?;
    Ok(Json(ProfileResponse::from(view)))
}

/// `PUT /api/v1/zenith/profile/me`
pub async fn put_me(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<UpdateProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    let view = state
        .update_profile
        .execute(UpdateProfileCommand {
            user_id: user.user_id,
            height: body.height,
            weight: body.weight,
            age: body.age,
        })
        .await?;
    Ok(Json(ProfileResponse::from(view)))
}
