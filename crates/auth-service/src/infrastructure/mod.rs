//! Инфраструктурный слой.
//!
//! Конкретные реализации портов прикладного слоя:
//! * `postgres` — [`AccountRepository`](crate::application::ports::AccountRepository) на SQLx;
//! * `redis`    — refresh-store и blacklist;
//! * `kafka`    — публикация событий;
//! * `security` — Argon2id-хешер и Ed25519 token service.

pub mod config;
pub mod kafka;
pub mod postgres;
pub mod redis;
pub mod security;
