//! Доменные ошибки Zenith.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DomainError {
    #[error("height must be between 0 and 300 cm")]
    InvalidHeight,
    #[error("weight must be between 0 and 1000 kg")]
    InvalidWeight,
    #[error("age must be between 0 and 150")]
    InvalidAge,
    #[error("food name must be 1-128 characters")]
    InvalidFoodName,
    #[error("calories must be between 0 and 100000")]
    InvalidCalories,
    #[error("workout kind must be 1-64 characters")]
    InvalidWorkoutKind,
    #[error("duration must be between 1 and 1440 minutes")]
    InvalidDuration,
}
