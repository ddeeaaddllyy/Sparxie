//! PostgreSQL-инфраструктура RequiemProject.

pub mod pool;
pub mod profile_repository;

pub use pool::{connect_pool, run_migrations};
pub use profile_repository::PgProfileRepository;
