//! Записи еды и тренировок.

use time::OffsetDateTime;
use uuid::Uuid;

use super::error::DomainError;

const MAX_CALORIES: i32 = 100_000;
const MAX_DURATION: i32 = 1_440; // минут в сутках

/// Запись о приёме пищи.
#[derive(Debug, Clone)]
pub struct FoodEntry {
    id: Uuid,
    user_id: Uuid,
    name: String,
    calories: i32,
    eaten_at: OffsetDateTime,
}

impl FoodEntry {
    /// Создаёт валидную запись (генерирует id).
    pub fn create(
        user_id: Uuid,
        name: String,
        calories: i32,
        eaten_at: OffsetDateTime,
    ) -> Result<Self, DomainError> {
        let name = name.trim().to_owned();
        if name.is_empty() || name.chars().count() > 128 {
            return Err(DomainError::InvalidFoodName);
        }
        if !(0..=MAX_CALORIES).contains(&calories) {
            return Err(DomainError::InvalidCalories);
        }
        Ok(Self {
            id: Uuid::new_v4(),
            user_id,
            name,
            calories,
            eaten_at,
        })
    }

    pub fn from_persistence(
        id: Uuid,
        user_id: Uuid,
        name: String,
        calories: i32,
        eaten_at: OffsetDateTime,
    ) -> Self {
        Self {
            id,
            user_id,
            name,
            calories,
            eaten_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn calories(&self) -> i32 {
        self.calories
    }
    pub fn eaten_at(&self) -> OffsetDateTime {
        self.eaten_at
    }
}

/// Запись о тренировке.
#[derive(Debug, Clone)]
pub struct WorkoutEntry {
    id: Uuid,
    user_id: Uuid,
    kind: String,
    duration_min: i32,
    performed_at: OffsetDateTime,
}

impl WorkoutEntry {
    pub fn create(
        user_id: Uuid,
        kind: String,
        duration_min: i32,
        performed_at: OffsetDateTime,
    ) -> Result<Self, DomainError> {
        let kind = kind.trim().to_owned();
        if kind.is_empty() || kind.chars().count() > 64 {
            return Err(DomainError::InvalidWorkoutKind);
        }
        if !(1..=MAX_DURATION).contains(&duration_min) {
            return Err(DomainError::InvalidDuration);
        }
        Ok(Self {
            id: Uuid::new_v4(),
            user_id,
            kind,
            duration_min,
            performed_at,
        })
    }

    pub fn from_persistence(
        id: Uuid,
        user_id: Uuid,
        kind: String,
        duration_min: i32,
        performed_at: OffsetDateTime,
    ) -> Self {
        Self {
            id,
            user_id,
            kind,
            duration_min,
            performed_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }
    pub fn kind(&self) -> &str {
        &self.kind
    }
    pub fn duration_min(&self) -> i32 {
        self.duration_min
    }
    pub fn performed_at(&self) -> OffsetDateTime {
        self.performed_at
    }
}
