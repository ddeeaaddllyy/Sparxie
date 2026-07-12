//! PostgreSQL-инфраструктура Zenith.

pub mod entry_repository;
pub mod pool;
pub mod profile_repository;

pub use entry_repository::PgEntryRepository;
pub use pool::{connect_pool, run_migrations};
pub use profile_repository::PgProfileRepository;
