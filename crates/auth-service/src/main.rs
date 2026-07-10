//! `nedovolen-auth` — центральный сервис авторизации и аккаунтов.
//!
//! Composition root: инициализация окружения/конфигурации/трассировки, сборка
//! инфраструктуры и зависимостей (DI), запуск HTTP-сервера.

mod application;
mod domain;
mod infrastructure;
mod presentation;

use std::sync::Arc;

use anyhow::Context;

use application::ports::{
    AccessTokenBlacklist, AccountRepository, EventPublisher, PasswordHasher, RefreshTokenStore,
    TokenService,
};
use application::use_cases::{
    ChangePasswordUseCase, DeleteAccountUseCase, GetAccountUseCase, LoginUseCase, LogoutUseCase,
    RefreshUseCase, RegisterUseCase,
};
use infrastructure::config::AuthConfig;
use infrastructure::kafka::KafkaEventPublisher;
use infrastructure::postgres::{self, PgAccountRepository};
use infrastructure::redis::{self as redis_infra, RedisAccessTokenBlacklist, RedisRefreshTokenStore};
use infrastructure::security::{Argon2PasswordHasher, Ed25519TokenService};
use presentation::http::{AppState, build_router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared::config::load_dotenv();
    shared::telemetry::init("nedovolen-auth", "info,auth_service=debug")
        .context("failed to initialize telemetry")?;

    let config = AuthConfig::from_env().context("failed to load configuration")?;
    let addr = config.socket_addr();

    let state = build_state(&config)
        .await
        .context("failed to build application state")?;

    let app = build_router(state, config.request_timeout);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    tracing::info!(%addr, "nedovolen-auth is listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

    tracing::info!("nedovolen-auth stopped");
    Ok(())
}

/// Собирает инфраструктуру и внедряет её в use cases (Dependency Injection).
async fn build_state(config: &AuthConfig) -> anyhow::Result<AppState> {
    // ── Внешние ресурсы ──────────────────────────────────────────────────────
    let pool = postgres::connect_pool(&config.database_url, 10)
        .await
        .context("connect to PostgreSQL")?;
    postgres::run_migrations(&pool)
        .await
        .context("run database migrations")?;

    let redis = redis_infra::connect(&config.redis_url)
        .await
        .context("connect to Redis")?;

    // ── Адаптеры портов ──────────────────────────────────────────────────────
    let accounts: Arc<dyn AccountRepository> = Arc::new(PgAccountRepository::new(pool.clone()));
    let hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher);
    let token_service: Arc<dyn TokenService> = Arc::new(
        Ed25519TokenService::from_files(
            &config.jwt.private_key_path,
            &config.jwt.public_key_path,
            config.jwt.issuer.clone(),
            config.jwt.access_ttl,
            config.jwt.refresh_ttl,
        )
        .context("load Ed25519 JWT keys")?,
    );
    let refresh_store: Arc<dyn RefreshTokenStore> =
        Arc::new(RedisRefreshTokenStore::new(redis.clone()));
    let blacklist: Arc<dyn AccessTokenBlacklist> =
        Arc::new(RedisAccessTokenBlacklist::new(redis.clone()));
    let events: Arc<dyn EventPublisher> = Arc::new(
        KafkaEventPublisher::new(&config.kafka_brokers, config.kafka_user_events_topic.clone())
            .context("create Kafka producer")?,
    );

    // ── Use cases ────────────────────────────────────────────────────────────
    let register = Arc::new(RegisterUseCase::new(
        accounts.clone(),
        hasher.clone(),
        token_service.clone(),
        refresh_store.clone(),
        events.clone(),
    ));
    let login = Arc::new(LoginUseCase::new(
        accounts.clone(),
        hasher.clone(),
        token_service.clone(),
        refresh_store.clone(),
        events.clone(),
    ));
    let refresh = Arc::new(RefreshUseCase::new(
        token_service.clone(),
        refresh_store.clone(),
    ));
    let logout = Arc::new(LogoutUseCase::new(
        token_service.clone(),
        refresh_store.clone(),
        blacklist.clone(),
    ));
    let change_password = Arc::new(ChangePasswordUseCase::new(
        accounts.clone(),
        hasher.clone(),
        refresh_store.clone(),
        events.clone(),
    ));
    let delete_account = Arc::new(DeleteAccountUseCase::new(
        accounts.clone(),
        hasher.clone(),
        refresh_store.clone(),
        events.clone(),
    ));
    let get_account = Arc::new(GetAccountUseCase::new(accounts.clone()));

    Ok(AppState {
        pool,
        redis,
        token_service,
        blacklist,
        register,
        login,
        refresh,
        logout,
        change_password,
        delete_account,
        get_account,
    })
}

/// Ожидает сигнал завершения (Ctrl+C или SIGTERM) для graceful shutdown.
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

    tracing::info!("shutdown signal received");
}
