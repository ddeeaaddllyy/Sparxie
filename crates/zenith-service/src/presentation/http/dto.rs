//! HTTP-DTO Zenith.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::application::dto::{FoodView, ProfileView, WorkoutView};

// ─── Профиль ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(range(min = 0, max = 300, message = "height must be 0-300"))]
    pub height: Option<i32>,
    #[validate(range(min = 0, max = 1000, message = "weight must be 0-1000"))]
    pub weight: Option<i32>,
    #[validate(range(min = 0, max = 150, message = "age must be 0-150"))]
    pub age: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub user_id: Uuid,
    pub height: i32,
    pub weight: i32,
    pub age: i32,
    pub streak: i32,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<ProfileView> for ProfileResponse {
    fn from(v: ProfileView) -> Self {
        Self {
            user_id: v.user_id,
            height: v.height,
            weight: v.weight,
            age: v.age,
            streak: v.streak,
            created_at: v.created_at,
        }
    }
}

// ─── Еда ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct AddFoodRequest {
    #[validate(length(min = 1, max = 128, message = "name must be 1-128 characters"))]
    pub name: String,
    #[validate(range(min = 0, max = 100_000, message = "calories must be 0-100000"))]
    pub calories: i32,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub eaten_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize)]
pub struct FoodResponse {
    pub id: Uuid,
    pub name: String,
    pub calories: i32,
    #[serde(with = "time::serde::rfc3339")]
    pub eaten_at: OffsetDateTime,
}

impl From<FoodView> for FoodResponse {
    fn from(v: FoodView) -> Self {
        Self {
            id: v.id,
            name: v.name,
            calories: v.calories,
            eaten_at: v.eaten_at,
        }
    }
}

// ─── Тренировки ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct AddWorkoutRequest {
    #[validate(length(min = 1, max = 64, message = "kind must be 1-64 characters"))]
    pub kind: String,
    #[validate(range(min = 1, max = 1440, message = "duration_min must be 1-1440"))]
    pub duration_min: i32,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub performed_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize)]
pub struct WorkoutResponse {
    pub id: Uuid,
    pub kind: String,
    pub duration_min: i32,
    #[serde(with = "time::serde::rfc3339")]
    pub performed_at: OffsetDateTime,
}

impl From<WorkoutView> for WorkoutResponse {
    fn from(v: WorkoutView) -> Self {
        Self {
            id: v.id,
            kind: v.kind,
            duration_min: v.duration_min,
            performed_at: v.performed_at,
        }
    }
}
