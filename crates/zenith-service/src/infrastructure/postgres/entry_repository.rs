//! Реализация [`EntryRepository`] на SQLx.

use async_trait::async_trait;
use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::application::ports::{EntryRepository, RepositoryError};
use crate::domain::{FoodEntry, WorkoutEntry};

/// Нарушение внешнего ключа (профиля-владельца ещё нет).
const PG_FK_VIOLATION: &str = "23503";

fn is_fk_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db) if db.code().as_deref() == Some(PG_FK_VIOLATION))
}

#[derive(Clone)]
pub struct PgEntryRepository {
    pool: PgPool,
}

impl PgEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EntryRepository for PgEntryRepository {
    async fn add_food(&self, entry: &FoodEntry) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            INSERT INTO food_entries (id, user_uuid, name, calories, eaten_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            entry.id(),
            entry.user_id(),
            entry.name(),
            entry.calories(),
            entry.eaten_at(),
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) if is_fk_violation(&e) => Err(RepositoryError::NotFound),
            Err(e) => Err(RepositoryError::Backend(e.into())),
        }
    }

    async fn list_food(&self, user_id: Uuid) -> Result<Vec<FoodEntry>, RepositoryError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_uuid, name, calories, eaten_at
            FROM food_entries
            WHERE user_uuid = $1
            ORDER BY eaten_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Backend(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|r| FoodEntry::from_persistence(r.id, r.user_uuid, r.name, r.calories, r.eaten_at))
            .collect())
    }

    async fn add_workout(&self, entry: &WorkoutEntry) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            INSERT INTO workout_entries (id, user_uuid, kind, duration_min, performed_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            entry.id(),
            entry.user_id(),
            entry.kind(),
            entry.duration_min(),
            entry.performed_at(),
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) if is_fk_violation(&e) => Err(RepositoryError::NotFound),
            Err(e) => Err(RepositoryError::Backend(e.into())),
        }
    }

    async fn list_workout(&self, user_id: Uuid) -> Result<Vec<WorkoutEntry>, RepositoryError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_uuid, kind, duration_min, performed_at
            FROM workout_entries
            WHERE user_uuid = $1
            ORDER BY performed_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Backend(e.into()))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                WorkoutEntry::from_persistence(
                    r.id,
                    r.user_uuid,
                    r.kind,
                    r.duration_min,
                    r.performed_at,
                )
            })
            .collect())
    }
}
