//! Обработчики записей тренировок.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use shared::web::{AuthUser, ValidatedJson};
use time::OffsetDateTime;

use crate::application::dto::AddWorkoutCommand;
use crate::presentation::http::AppState;
use crate::presentation::http::dto::{AddWorkoutRequest, WorkoutResponse};
use crate::presentation::http::error::AppError;

/// `POST /api/v1/zenith/workout`
pub async fn add(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<AddWorkoutRequest>,
) -> Result<impl IntoResponse, AppError> {
    let view = state
        .add_workout
        .execute(AddWorkoutCommand {
            user_id: user.user_id,
            kind: body.kind,
            duration_min: body.duration_min,
            performed_at: body.performed_at.unwrap_or_else(OffsetDateTime::now_utc),
        })
        .await?;
    Ok((StatusCode::CREATED, Json(WorkoutResponse::from(view))))
}

/// `GET /api/v1/zenith/workout`
pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let items = state.list_workout.execute(user.user_id).await?;
    let response: Vec<WorkoutResponse> = items.into_iter().map(WorkoutResponse::from).collect();
    Ok(Json(response))
}
