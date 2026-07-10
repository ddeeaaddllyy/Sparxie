//! Реализация [`AccountRepository`] на SQLx с compile-time проверкой запросов.

use async_trait::async_trait;
use sqlx::postgres::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::application::ports::{AccountRepository, RepositoryError};
use crate::domain::{Account, Nickname, PasswordHash, UserId};

/// Хранилище аккаунтов в PostgreSQL.
#[derive(Clone)]
pub struct PgAccountRepository {
    pool: PgPool,
}

impl PgAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Код ошибки нарушения уникальности в PostgreSQL.
const PG_UNIQUE_VIOLATION: &str = "23505";

fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db) if db.code().as_deref() == Some(PG_UNIQUE_VIOLATION))
}

/// Реконструирует доменную сущность из строки БД.
///
/// Данные в БД считаются доверенными; невозможность распарсить value-объект
/// означает повреждение данных → внутренняя ошибка.
fn map_account_row(
    uuid: Uuid,
    nickname: String,
    password_hash: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
) -> Result<Account, RepositoryError> {
    let nickname = Nickname::parse(nickname)
        .map_err(|e| RepositoryError::Backend(anyhow::anyhow!("corrupt nickname in db: {e}")))?;
    let password_hash = PasswordHash::from_hash(password_hash)
        .map_err(|e| RepositoryError::Backend(anyhow::anyhow!("corrupt password hash in db: {e}")))?;

    Ok(Account::from_persistence(
        UserId::from_uuid(uuid),
        nickname,
        password_hash,
        created_at,
        updated_at,
    ))
}

#[async_trait]
impl AccountRepository for PgAccountRepository {
    async fn find_by_id(&self, id: UserId) -> Result<Option<Account>, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT uuid, nickname, password_hash, created_at, updated_at
            FROM accounts
            WHERE uuid = $1
            "#,
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Backend(e.into()))?;

        row.map(|r| {
            map_account_row(r.uuid, r.nickname, r.password_hash, r.created_at, r.updated_at)
        })
        .transpose()
    }

    async fn find_by_nickname(
        &self,
        nickname: &Nickname,
    ) -> Result<Option<Account>, RepositoryError> {
        let row = sqlx::query!(
            r#"
            SELECT uuid, nickname, password_hash, created_at, updated_at
            FROM accounts
            WHERE nickname = $1
            "#,
            nickname.as_str()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Backend(e.into()))?;

        row.map(|r| {
            map_account_row(r.uuid, r.nickname, r.password_hash, r.created_at, r.updated_at)
        })
        .transpose()
    }

    async fn exists_by_nickname(&self, nickname: &Nickname) -> Result<bool, RepositoryError> {
        let row = sqlx::query!(
            r#"SELECT EXISTS(SELECT 1 FROM accounts WHERE nickname = $1) AS "exists!""#,
            nickname.as_str()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| RepositoryError::Backend(e.into()))?;

        Ok(row.exists)
    }

    async fn insert(&self, account: &Account) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            INSERT INTO accounts (uuid, nickname, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            account.id().as_uuid(),
            account.nickname().as_str(),
            account.password_hash().as_str(),
            account.created_at(),
            account.updated_at(),
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) if is_unique_violation(&e) => Err(RepositoryError::NicknameConflict),
            Err(e) => Err(RepositoryError::Backend(e.into())),
        }
    }

    async fn update_password(
        &self,
        id: UserId,
        password_hash: &PasswordHash,
    ) -> Result<(), RepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE accounts
            SET password_hash = $2, updated_at = now()
            WHERE uuid = $1
            "#,
            id.as_uuid(),
            password_hash.as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Backend(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }

    async fn delete(&self, id: UserId) -> Result<(), RepositoryError> {
        let result = sqlx::query!(r#"DELETE FROM accounts WHERE uuid = $1"#, id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| RepositoryError::Backend(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domain::{Account, Nickname, PasswordHash};
    use crate::infrastructure::postgres::pool::connect_pool;

    /// Полный CRUD-цикл против живой БД.
    /// Требует `DATABASE_URL`; запуск: `cargo test -- --ignored`.
    #[tokio::test]
    #[ignore = "requires DATABASE_URL to a live PostgreSQL"]
    async fn crud_roundtrip() {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = connect_pool(&url, 2).await.expect("connect");
        let repo = PgAccountRepository::new(pool);

        let suffix = Uuid::new_v4().simple().to_string();
        let nickname = Nickname::parse(format!("t_{}", &suffix[..10])).unwrap();
        let account = Account::register(
            nickname.clone(),
            PasswordHash::from_hash("$argon2id$test$dummyhash").unwrap(),
        );
        let id = account.id();

        // insert + чтения
        repo.insert(&account).await.unwrap();
        assert!(repo.exists_by_nickname(&nickname).await.unwrap());
        assert!(repo.find_by_id(id).await.unwrap().is_some());
        assert!(repo.find_by_nickname(&nickname).await.unwrap().is_some());

        // повтор никнейма → конфликт
        let dup = Account::register(
            nickname.clone(),
            PasswordHash::from_hash("$argon2id$test$other").unwrap(),
        );
        assert!(matches!(
            repo.insert(&dup).await,
            Err(RepositoryError::NicknameConflict)
        ));

        // обновление пароля
        repo.update_password(id, &PasswordHash::from_hash("$argon2id$test$new").unwrap())
            .await
            .unwrap();

        // удаление + отсутствие
        repo.delete(id).await.unwrap();
        assert!(repo.find_by_id(id).await.unwrap().is_none());
        assert!(matches!(
            repo.delete(id).await,
            Err(RepositoryError::NotFound)
        ));
    }
}
