//! Сборка Axum-роутера, разделение публичных/защищённых маршрутов и middleware.

use std::time::Duration;

use axum::http::StatusCode;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, patch, post};
use axum::Router;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use super::handlers::{accounts, auth, health};
use super::middleware::jwt_auth;
use super::state::AppState;

/// Строит корневой роутер сервиса.
///
/// Глобальный стек middleware (сверху вниз к входящему запросу):
/// Request ID → Trace → Propagate ID → Compression → CORS → Timeout.
/// Защищённые маршруты дополнительно проходят JWT-middleware.
pub fn build_router(state: AppState, request_timeout: Duration) -> Router {
    let global = ServiceBuilder::new()
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            request_timeout,
        ));

    // Публичные маршруты (без аутентификации).
    let public = Router::new()
        .route("/healthz", get(health::liveness))
        .route("/readyz", get(health::readiness))
        .route("/api/v1/auth/register", post(auth::register))
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/refresh", post(auth::refresh));

    // Защищённые маршруты (требуют валидный access-токен).
    let protected = Router::new()
        .route("/api/v1/auth/logout", post(auth::logout))
        .route("/api/v1/accounts/me", get(accounts::me).delete(accounts::delete_me))
        .route("/api/v1/accounts/me/password", patch(accounts::change_password))
        .route_layer(from_fn_with_state(state.clone(), jwt_auth));

    public
        .merge(protected)
        .layer(global)
        .with_state(state)
}
