//! Доменный слой Zenith.

pub mod entries;
pub mod error;
pub mod profile;

pub use entries::{FoodEntry, WorkoutEntry};
pub use error::DomainError;
pub use profile::ZenithProfile;
