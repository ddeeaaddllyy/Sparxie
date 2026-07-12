//! Прикладные команды и представления Zenith.

use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::{FoodEntry, WorkoutEntry, ZenithProfile};

#[derive(Debug)]
pub struct UpdateProfileCommand {
    pub user_id: Uuid,
    pub height: Option<i32>,
    pub weight: Option<i32>,
    pub age: Option<i32>,
}

#[derive(Debug)]
pub struct AddFoodCommand {
    pub user_id: Uuid,
    pub name: String,
    pub calories: i32,
    pub eaten_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct AddWorkoutCommand {
    pub user_id: Uuid,
    pub kind: String,
    pub duration_min: i32,
    pub performed_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct ProfileView {
    pub user_id: Uuid,
    pub height: i32,
    pub weight: i32,
    pub age: i32,
    pub streak: i32,
    pub created_at: OffsetDateTime,
}

impl From<&ZenithProfile> for ProfileView {
    fn from(p: &ZenithProfile) -> Self {
        Self {
            user_id: p.user_id(),
            height: p.height(),
            weight: p.weight(),
            age: p.age(),
            streak: p.streak(),
            created_at: p.created_at(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FoodView {
    pub id: Uuid,
    pub name: String,
    pub calories: i32,
    pub eaten_at: OffsetDateTime,
}

impl From<&FoodEntry> for FoodView {
    fn from(e: &FoodEntry) -> Self {
        Self {
            id: e.id(),
            name: e.name().to_owned(),
            calories: e.calories(),
            eaten_at: e.eaten_at(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkoutView {
    pub id: Uuid,
    pub kind: String,
    pub duration_min: i32,
    pub performed_at: OffsetDateTime,
}

impl From<&WorkoutEntry> for WorkoutView {
    fn from(e: &WorkoutEntry) -> Self {
        Self {
            id: e.id(),
            kind: e.kind().to_owned(),
            duration_min: e.duration_min(),
            performed_at: e.performed_at(),
        }
    }
}
