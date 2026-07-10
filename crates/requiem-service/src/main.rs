//! `requiem-service` — сервис профилей RequiemProject.
//!
//! Consumer событий `nedovolen.user.events` (создание/удаление профиля по UUID)
//! + защищённый JWT REST-API профиля. Учётные данные не хранит.

mod application;
mod domain;
mod infrastructure;
mod presentation;

use std::sync::Arc;

use anyhow::Context;
use shared::jwt::AccessTokenVerifier;

use application::ports::ProfileRepository;
use application::use_cases::{ApplyUserDeleted, ApplyUserRegistered, GetProfile, UpdateProfile};
use infrastructure::config::RequiemConfig;
use infrastructure::kafka::EventConsumer;
use infrastructure::postgres::{PgProfileRepository, connect_pool, run_migrations};
use presentation::http::{AppState, build_router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared::config::load_dotenv();
    shared::telemetry::init("requiem-service", "info,requiem_service=debug")
        .context("failed to initialize telemetry")?;

    let config = RequiemConfig::from_env().context("failed to load configuration")?;

    let pool = connect_pool(&config.database_url, 10)
        .await
        .context("connect to PostgreSQL")?;
    run_migrations(&pool).await.context("run migrations")?;

    let repo: Arc<dyn ProfileRepository> = Arc::new(PgProfileRepository::new(pool.clone()));
    let on_registered = Arc::new(ApplyUserRegistered::new(repo.clone()));
    let on_deleted = Arc::new(ApplyUserDeleted::new(repo.clone()));
    let get_profile = Arc::new(GetProfile::new(repo.clone()));
    let update_profile = Arc::new(UpdateProfile::new(repo.clone()));

    let verifier = Arc::new(
        AccessTokenVerifier::from_file(&config.jwt.public_key_path, &config.jwt.issuer)
            .context("load JWT public key")?,
    );

    // Kafka-консюмер в отдельной задаче.
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
        get_profile,
        update_profile,
    };

    let addr = config.socket_addr();
    let app = build_router(state, config.request_timeout);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    tracing::info!(%addr, "requiem-service is listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    consumer_handle.abort();
    tracing::info!("requiem-service stopped");
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
