//! PostgreSQL-инфраструктура: пул соединений и репозиторий аккаунтов.

pub mod account_repository;
pub mod pool;

pub use account_repository::PgAccountRepository;
pub use pool::{connect_pool, run_migrations};
