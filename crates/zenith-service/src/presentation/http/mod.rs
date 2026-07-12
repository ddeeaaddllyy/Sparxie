//! HTTP-подслой Zenith.

pub mod dto;
pub mod error;
pub mod handlers;
pub mod routes;
pub mod state;

pub use routes::build_router;
pub use state::AppState;
