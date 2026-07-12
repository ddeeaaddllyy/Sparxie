//! Обработчики записей еды.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use shared::web::{AuthUser, ValidatedJson};
use time::OffsetDateTime;

use crate::application::dto::AddFoodCommand;
use crate::presentation::http::AppState;
use crate::presentation::http::dto::{AddFoodRequest, FoodResponse};
use crate::presentation::http::error::AppError;

/// `POST /api/v1/zenith/food`
pub async fn add(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<AddFoodRequest>,
) -> Result<impl IntoResponse, AppError> {
    let view = state
        .add_food
        .execute(AddFoodCommand {
            user_id: user.user_id,
            name: body.name,
            calories: body.calories,
            eaten_at: body.eaten_at.unwrap_or_else(OffsetDateTime::now_utc),
        })
        .await?;
    Ok((StatusCode::CREATED, Json(FoodResponse::from(view))))
}

/// `GET /api/v1/zenith/food`
pub async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let items = state.list_food.execute(user.user_id).await?;
    let response: Vec<FoodResponse> = items.into_iter().map(FoodResponse::from).collect();
    Ok(Json(response))
}
