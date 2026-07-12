//! `AppState` — DI-контейнер zenith-service.

use std::sync::Arc;

use shared::jwt::AccessTokenVerifier;
use shared::web::HasAccessVerifier;
use sqlx::postgres::PgPool;

use crate::application::use_cases::{
    AddFood, AddWorkout, GetProfile, ListFood, ListWorkout, UpdateProfile,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub verifier: Arc<AccessTokenVerifier>,
    pub get_profile: Arc<GetProfile>,
    pub update_profile: Arc<UpdateProfile>,
    pub add_food: Arc<AddFood>,
    pub list_food: Arc<ListFood>,
    pub add_workout: Arc<AddWorkout>,
    pub list_workout: Arc<ListWorkout>,
}

impl HasAccessVerifier for AppState {
    fn access_verifier(&self) -> &AccessTokenVerifier {
        self.verifier.as_ref()
    }
}
