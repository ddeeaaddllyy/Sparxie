//! Реестр активных refresh-токенов (whitelist) в Redis.
//!
//! Схема ключей: `refresh:{user_id}:{jti}` со значением `"1"` и TTL, равным
//! сроку жизни refresh-токена. Наличие ключа = токен действителен.

use std::time::Duration;

use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use uuid::Uuid;

use crate::application::ports::{RefreshStoreError, RefreshTokenStore};
use crate::domain::UserId;

#[derive(Clone)]
pub struct RedisRefreshTokenStore {
    manager: ConnectionManager,
}

impl RedisRefreshTokenStore {
    pub fn new(manager: ConnectionManager) -> Self {
        Self { manager }
    }

    fn key(user_id: UserId, jti: Uuid) -> String {
        format!("refresh:{}:{}", user_id.as_uuid(), jti)
    }

    fn user_pattern(user_id: UserId) -> String {
        format!("refresh:{}:*", user_id.as_uuid())
    }
}

#[async_trait]
impl RefreshTokenStore for RedisRefreshTokenStore {
    async fn store(
        &self,
        user_id: UserId,
        jti: Uuid,
        ttl: Duration,
    ) -> Result<(), RefreshStoreError> {
        let secs = ttl.as_secs();
        if secs == 0 {
            // Токен уже истёк
            return Ok(());
        }

        let mut conn = self.manager.clone();
        let _: () = conn
            .set_ex(Self::key(user_id, jti), 1u8, secs)
            .await
            .map_err(|e| RefreshStoreError::Backend(e.into()))?;
        Ok(())
    }

    async fn is_active(&self, user_id: UserId, jti: Uuid) -> Result<bool, RefreshStoreError> {
        let mut conn = self.manager.clone();
        let exists: bool = conn
            .exists(Self::key(user_id, jti))
            .await
            .map_err(|e| RefreshStoreError::Backend(e.into()))?;
        Ok(exists)
    }

    async fn revoke(&self, user_id: UserId, jti: Uuid) -> Result<(), RefreshStoreError> {
        let mut conn = self.manager.clone();
        let _: () = conn
            .del(Self::key(user_id, jti))
            .await
            .map_err(|e| RefreshStoreError::Backend(e.into()))?;
        Ok(())
    }

    async fn revoke_all(&self, user_id: UserId) -> Result<(), RefreshStoreError> {
        let mut conn = self.manager.clone();
        let pattern = Self::user_pattern(user_id);

        // Итеративный SCAN (без блокирующего KEYS): собираем ключи, затем DEL.
        let mut cursor: u64 = 0;
        let mut keys: Vec<String> = Vec::new();
        loop {
            let (next, batch): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .map_err(|e| RefreshStoreError::Backend(e.into()))?;
            keys.extend(batch);
            if next == 0 {
                break;
            }
            cursor = next;
        }

        if !keys.is_empty() {
            let _: () = conn
                .del(keys)
                .await
                .map_err(|e| RefreshStoreError::Backend(e.into()))?;
        }
        Ok(())
    }
}
