//! Установка соединения с Redis (через мультиплексируемый `ConnectionManager`).

use redis::aio::ConnectionManager;

/// Создаёт `ConnectionManager` — клонируемый, автоматически переподключающийся
/// мультиплексируемый клиент.
pub async fn connect(url: &str) -> anyhow::Result<ConnectionManager> {
    let client = redis::Client::open(url)?;
    let manager = ConnectionManager::new(client).await?;
    Ok(manager)
}
