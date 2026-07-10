//! Сборка роутера RequiemProject.

use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use shared::web::jwt_auth;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use super::handlers::{health, profile};
use super::state::AppState;

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

    let public = Router::new()
        .route("/healthz", get(health::liveness))
        .route("/readyz", get(health::readiness));

    let protected = Router::new()
        .route(
            "/api/v1/requiem/profile/me",
            get(profile::get_me).put(profile::put_me),
        )
        .route_layer(from_fn_with_state(state.clone(), jwt_auth::<AppState>));

    public.merge(protected).layer(global).with_state(state)
}
