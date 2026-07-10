//! Инициализация трассировки (`tracing`).
//!
//! Формат логов управляется переменной `LOG_FORMAT` (`json` | `pretty`),
//! уровень — стандартной `RUST_LOG` (через `EnvFilter`).

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

/// Настраивает глобального подписчика трассировки.
///
/// Вызывается один раз при старте процесса. Повторный вызов вернёт ошибку —
/// её обрабатывает вызывающая сторона (`main`).
///
/// * `service_name` — попадает в поле `service` каждого JSON-лога.
/// * `default_directive` — уровень по умолчанию, если `RUST_LOG` не задан.
pub fn init(service_name: &'static str, default_directive: &str) -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(default_directive))
        .map_err(|e| anyhow::anyhow!("failed to build tracing EnvFilter: {e}"))?;

    let use_json = std::env::var("LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(true);

    let registry = tracing_subscriber::registry().with(env_filter);

    if use_json {
        registry
            .with(
                fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_target(true),
            )
            .try_init()
            .map_err(|e| anyhow::anyhow!("failed to init tracing subscriber: {e}"))?;
    } else {
        registry
            .with(fmt::layer().with_target(true))
            .try_init()
            .map_err(|e| anyhow::anyhow!("failed to init tracing subscriber: {e}"))?;
    }

    tracing::debug!(service = service_name, "tracing initialized");
    Ok(())
}
