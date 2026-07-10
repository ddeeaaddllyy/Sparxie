//! `AppState` — контейнер зависимостей (Dependency Injection через Axum `State`).
//!
//! Хранит собранные use cases (за портами `Arc<dyn …>`) и «сырые» ресурсы,
//! нужные инфраструктурным слоям представления (readiness-проверки, JWT-middleware).

use std::sync::Arc;

use redis::aio::ConnectionManager;
use sqlx::postgres::PgPool;

use crate::application::ports::{AccessTokenBlacklist, TokenService};
use crate::application::use_cases::{
    ChangePasswordUseCase, DeleteAccountUseCase, GetAccountUseCase, LoginUseCase, LogoutUseCase,
    RefreshUseCase, RegisterUseCase,
};

/// Разделяемое состояние приложения. Дёшево клонируется (внутри — `Arc`/пулы).
#[derive(Clone)]
pub struct AppState {
    // Ресурсы для readiness-проверок.
    pub pool: PgPool,
    pub redis: ConnectionManager,

    // Нужны JWT-middleware (этап 4): проверка подписи и blacklist.
    pub token_service: Arc<dyn TokenService>,
    pub blacklist: Arc<dyn AccessTokenBlacklist>,

    // Use cases.
    pub register: Arc<RegisterUseCase>,
    pub login: Arc<LoginUseCase>,
    pub refresh: Arc<RefreshUseCase>,
    pub logout: Arc<LogoutUseCase>,
    pub change_password: Arc<ChangePasswordUseCase>,
    pub delete_account: Arc<DeleteAccountUseCase>,
    pub get_account: Arc<GetAccountUseCase>,
}
