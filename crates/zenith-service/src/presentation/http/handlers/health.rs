//! Health-check эндпоинты.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;

use crate::presentation::http::AppState;

#[derive(Serialize)]
pub struct Health {
    pub status: &'static str,
    pub postgres: bool,
}

pub async fn liveness() -> Json<Health> {
    Json(Health {
        status: "ok",
        postgres: true,
    })
}

pub async fn readiness(State(state): State<AppState>) -> (StatusCode, Json<Health>) {
    let postgres = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .is_ok();

    let code = if postgres {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        code,
        Json(Health {
            status: if postgres { "ready" } else { "unavailable" },
            postgres,
        }),
    )
}
