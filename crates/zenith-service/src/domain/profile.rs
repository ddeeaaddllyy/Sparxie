//! Сущность `ZenithProfile`.

use time::OffsetDateTime;
use uuid::Uuid;

use super::error::DomainError;

const MAX_HEIGHT: i32 = 300;
const MAX_WEIGHT: i32 = 1000;
const MAX_AGE: i32 = 150;

/// Профиль пользователя Zenith (антропометрия + streak).
#[derive(Debug, Clone)]
pub struct ZenithProfile {
    user_id: Uuid,
    height: i32,
    weight: i32,
    age: i32,
    streak: i32,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl ZenithProfile {
    pub fn from_persistence(
        user_id: Uuid,
        height: i32,
        weight: i32,
        age: i32,
        streak: i32,
        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
    ) -> Self {
        Self {
            user_id,
            height,
            weight,
            age,
            streak,
            created_at,
            updated_at,
        }
    }

    /// Частичное обновление антропометрии с проверкой диапазонов.
    pub fn apply_update(
        &mut self,
        height: Option<i32>,
        weight: Option<i32>,
        age: Option<i32>,
    ) -> Result<(), DomainError> {
        if let Some(h) = height {
            if !(0..=MAX_HEIGHT).contains(&h) {
                return Err(DomainError::InvalidHeight);
            }
            self.height = h;
        }
        if let Some(w) = weight {
            if !(0..=MAX_WEIGHT).contains(&w) {
                return Err(DomainError::InvalidWeight);
            }
            self.weight = w;
        }
        if let Some(a) = age {
            if !(0..=MAX_AGE).contains(&a) {
                return Err(DomainError::InvalidAge);
            }
            self.age = a;
        }
        self.updated_at = OffsetDateTime::now_utc();
        Ok(())
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn weight(&self) -> i32 {
        self.weight
    }
    pub fn age(&self) -> i32 {
        self.age
    }
    pub fn streak(&self) -> i32 {
        self.streak
    }
    pub fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
}
