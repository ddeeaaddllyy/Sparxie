//! Создание пула соединений и применение миграций.

use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};

/// Создаёт пул соединений к PostgreSQL.
pub async fn connect_pool(database_url: &str, max_connections: u32) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// Применяет встроенные (compile-time) миграции из `./migrations`.
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
