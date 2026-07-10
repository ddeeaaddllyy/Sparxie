//! `AppState` — DI-контейнер requiem-service.

use std::sync::Arc;

use shared::jwt::AccessTokenVerifier;
use shared::web::HasAccessVerifier;
use sqlx::postgres::PgPool;

use crate::application::use_cases::{GetProfile, UpdateProfile};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub verifier: Arc<AccessTokenVerifier>,
    pub get_profile: Arc<GetProfile>,
    pub update_profile: Arc<UpdateProfile>,
}

impl HasAccessVerifier for AppState {
    fn access_verifier(&self) -> &AccessTokenVerifier {
        self.verifier.as_ref()
    }
}
