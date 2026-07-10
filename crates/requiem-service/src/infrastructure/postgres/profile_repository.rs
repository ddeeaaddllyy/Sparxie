//! Реализация [`ProfileRepository`] на SQLx (compile-time проверка запросов).

use async_trait::async_trait;
use sqlx::postgres::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::application::ports::{ProfileRepository, RepositoryError};
use crate::domain::{DisplayName, Email, RequiemProfile};

const PG_UNIQUE_VIOLATION: &str = "23505";

fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db) if db.code().as_deref() == Some(PG_UNIQUE_VIOLATION))
}

#[derive(Clone)]
pub struct PgProfileRepository {
    pool: PgPool,
}

impl PgProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn map_row(
    uuid: Uuid,
    email: Option<String>,
    display_name: Option<String>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
) -> Result<RequiemProfile, RepositoryError> {
    let email = email
        .map(Email::parse)
        .transpose()
        .map_err(|e| RepositoryError::Backend(anyhow::anyhow!("corrupt email in db: {e}")))?;
    let display_name = display_name
        .map(DisplayName::parse)
        .transpose()
        .map_err(|e| RepositoryError::Backend(anyhow::anyhow!("corrupt display_name in db: {e}")))?;

    Ok(RequiemProfile::from_persistence(
        uuid,
        email,
        display_name,
        created_at,
        updated_at,
    ))
}

#[async_trait]
impl ProfileRepository for PgProfileRepository {
    async fn upsert_shell(&self, user_id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO requiem_profiles (uuid)
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

    async fn find(&self, user_id: Uuid) -> Result<Option<RequiemProfile>, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT uuid, email, display_name, created_at, updated_at
            FROM requiem_profiles
            WHERE uuid = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Backend(e.into()))?;

        row.map(|r| map_row(r.uuid, r.email, r.display_name, r.created_at, r.updated_at))
            .transpose()
    }

    async fn save(&self, profile: &RequiemProfile) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE requiem_profiles
            SET email = $2, display_name = $3, updated_at = now()
            WHERE uuid = $1
            "#,
            profile.user_id(),
            profile.email().map(|e| e.as_str()),
            profile.display_name().map(|d| d.as_str()),
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(r) if r.rows_affected() == 0 => Err(RepositoryError::NotFound),
            Ok(_) => Ok(()),
            Err(e) if is_unique_violation(&e) => Err(RepositoryError::EmailConflict),
            Err(e) => Err(RepositoryError::Backend(e.into())),
        }
    }

    async fn delete(&self, user_id: Uuid) -> Result<(), RepositoryError> {
        // Идемпотентно: отсутствие строки не ошибка (событие могло прийти дважды).
        sqlx::query!(r#"DELETE FROM requiem_profiles WHERE uuid = $1"#, user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Backend(e.into()))?;
        Ok(())
    }
}
