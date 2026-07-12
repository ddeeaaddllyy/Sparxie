//! `zenith-service` — сервис данных Zenith (антропометрия, еда, тренировки).
//!
//! Consumer событий `nedovolen.user.events` + защищённый JWT REST-API.
//! Учётные данные не хранит.

mod application;
mod domain;
mod infrastructure;
mod presentation;

use std::sync::Arc;

use anyhow::Context;
use shared::jwt::AccessTokenVerifier;

use application::ports::{EntryRepository, ProfileRepository};
use application::use_cases::{
    AddFood, AddWorkout, ApplyUserDeleted, ApplyUserRegistered, GetProfile, ListFood, ListWorkout,
    UpdateProfile,
};
use infrastructure::config::ZenithConfig;
use infrastructure::kafka::EventConsumer;
use infrastructure::postgres::{
    PgEntryRepository, PgProfileRepository, connect_pool, run_migrations,
};
use presentation::http::{AppState, build_router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared::config::load_dotenv();
    shared::telemetry::init("zenith-service", "info,zenith_service=debug")
        .context("failed to initialize telemetry")?;

    let config = ZenithConfig::from_env().context("failed to load configuration")?;

    let pool = connect_pool(&config.database_url, 10)
        .await
        .context("connect to PostgreSQL")?;
    run_migrations(&pool).await.context("run migrations")?;

    let profiles: Arc<dyn ProfileRepository> = Arc::new(PgProfileRepository::new(pool.clone()));
    let entries: Arc<dyn EntryRepository> = Arc::new(PgEntryRepository::new(pool.clone()));

    let on_registered = Arc::new(ApplyUserRegistered::new(profiles.clone()));
    let on_deleted = Arc::new(ApplyUserDeleted::new(profiles.clone()));

    let verifier = Arc::new(
        AccessTokenVerifier::from_file(&config.jwt.public_key_path, &config.jwt.issuer)
            .context("load JWT public key")?,
    );

    let consumer = EventConsumer::new(&config.kafka, on_registered, on_deleted)
        .context("create Kafka consumer")?;
    let consumer_handle = tokio::spawn(async move {
        if let Err(e) = consumer.run().await {
            tracing::error!(error = %e, "kafka consumer terminated");
        }
    });

    let state = AppState {
        pool,
        verifier,
        get_profile: Arc::new(GetProfile::new(profiles.clone())),
        update_profile: Arc::new(UpdateProfile::new(profiles.clone())),
        add_food: Arc::new(AddFood::new(entries.clone())),
        list_food: Arc::new(ListFood::new(entries.clone())),
        add_workout: Arc::new(AddWorkout::new(entries.clone())),
        list_workout: Arc::new(ListWorkout::new(entries.clone())),
    };

    let addr = config.socket_addr();
    let app = build_router(state, config.request_timeout);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    tracing::info!(%addr, "zenith-service is listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    consumer_handle.abort();
    tracing::info!("zenith-service stopped");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
