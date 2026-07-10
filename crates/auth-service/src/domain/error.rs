//! Ошибки нарушения доменных инвариантов.
//!
//! Возникают исключительно при конструировании value-объектов из «сырых»
//! данных. Не содержат ничего инфраструктурного.

/// Нарушение инварианта предметной области.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    #[error("nickname must be 3–32 characters and contain only latin letters, digits, '_' or '-'")]
    InvalidNickname,

    #[error("password must be between 8 and 128 characters")]
    WeakPassword,

    #[error("password hash is empty or malformed")]
    InvalidPasswordHash,
}
