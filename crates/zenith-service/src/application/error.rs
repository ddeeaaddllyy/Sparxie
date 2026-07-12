//! Ошибки прикладного слоя Zenith.

use crate::application::ports::RepositoryError;
use crate::domain::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("validation error: {0}")]
    Validation(#[from] DomainError),

    #[error("profile not found")]
    NotFound,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<RepositoryError> for ApplicationError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound => ApplicationError::NotFound,
            RepositoryError::Backend(e) => ApplicationError::Internal(e),
        }
    }
}
