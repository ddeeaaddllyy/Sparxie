//! Доменные ошибки RequiemProject.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    #[error("invalid email address")]
    InvalidEmail,

    #[error("display name must be 1-64 characters")]
    InvalidDisplayName,
}
