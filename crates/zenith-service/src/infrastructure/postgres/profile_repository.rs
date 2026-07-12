//! Реализация [`ProfileRepository`] на SQLx.

use async_trait::async_trait;
use sqlx::postgres::PgPool;
use uuid::Uuid;

use crate::application::ports::{ProfileRepository, RepositoryError};
use crate::domain::ZenithProfile;

#[derive(Clone)]
pub struct PgProfileRepository {
    pool: PgPool,
}

impl PgProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProfileRepository for PgProfileRepository {
    async fn upsert_shell(&self, user_id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO zenith_profiles (uuid)
            VALUES ($1)
            ON CONFLICT (uuid) DO NOTHING
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Backend(e.into()))?;
        Ok(())
    }

    async fn find(&self, user_id: Uuid) -> Result<Option<ZenithProfile>, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT uuid, height, weight, age, streak, created_at, updated_at
            FROM zenith_profiles
            WHERE uuid = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Backend(e.into()))?;

        Ok(row.map(|r| {
            ZenithProfile::from_persistence(
                r.uuid, r.height, r.weight, r.age, r.streak, r.created_at, r.updated_at,
            )
        }))
    }

    async fn save(&self, profile: &ZenithProfile) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE zenith_profiles
            SET height = $2, weight = $3, age = $4, updated_at = now()
            WHERE uuid = $1
            "#,
            profile.user_id(),
            profile.height(),
            profile.weight(),
            profile.age(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Backend(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn delete(&self, user_id: Uuid) -> Result<(), RepositoryError> {
        // Идемпотентно; записи еды/тренировок удалятся каскадно (ON DELETE CASCADE).
        sqlx::query!(r#"DELETE FROM zenith_profiles WHERE uuid = $1"#, user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Backend(e.into()))?;
        Ok(())
    }
}
