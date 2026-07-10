//! HTTP-подслой: обработчики, DTO, экстракторы, middleware, состояние и роутер.

pub mod dto;
pub mod error;
pub mod extract;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod state;

pub use routes::build_router;
pub use state::AppState;
