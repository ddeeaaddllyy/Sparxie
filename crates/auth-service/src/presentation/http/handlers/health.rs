//! Health-check эндпоинты для оркестратора (liveness/readiness).

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Serialize;

use crate::presentation::http::AppState;

#[derive(Debug, Serialize)]
pub struct LivenessResponse {
    pub status: &'static str,
}

/// Liveness: процесс жив и способен отвечать.
pub async fn liveness() -> Json<LivenessResponse> {
    Json(LivenessResponse { status: "ok" })
}

#[derive(Debug, Serialize)]
pub struct ReadinessResponse {
    pub status: &'static str,
    pub postgres: bool,
    pub redis: bool,
}

/// Readiness: готовность обслуживать трафик — проверяет доступность
/// PostgreSQL и Redis.
///
/// Kafka намеренно не гейтит readiness: продюсер буферизует сообщения, а сбои
/// доставки наблюдаются через ошибки/логи публикации (иначе кратковременная
/// недоступность брокера выводила бы сервис из ротации).
pub async fn readiness(State(state): State<AppState>) -> (StatusCode, Json<ReadinessResponse>) {
    let postgres = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .is_ok();

    let redis = {
        let mut conn = state.redis.clone();
        redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .map(|pong| pong == "PONG")
            .unwrap_or(false)
    };

    if postgres && redis {
        (
            StatusCode::OK,
            Json(ReadinessResponse {
                status: "ready",
                postgres,
                redis,
            }),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ReadinessResponse {
                status: "unavailable",
                postgres,
                redis,
            }),
        )
    }
}
