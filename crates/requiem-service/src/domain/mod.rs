//! Доменный слой RequiemProject.

pub mod error;
pub mod profile;
pub mod value_objects;

pub use error::DomainError;
pub use profile::RequiemProfile;
pub use value_objects::{DisplayName, Email};
