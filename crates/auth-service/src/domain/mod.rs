//! Доменный слой.
//!
//! Самый внутренний слой Clean Architecture: сущности, value-объекты и их
//! инварианты. Не зависит ни от Axum, ни от SQLx, ни от Kafka — только чистый
//! Rust (+ `uuid`/`time` как примитивы предметной области).

pub mod account;
pub mod error;
pub mod value_objects;

pub use account::Account;
pub use error::DomainError;
pub use value_objects::{Nickname, Password, PasswordHash, UserId};
