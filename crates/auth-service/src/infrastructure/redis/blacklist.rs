//! Blacklist access-токенов в Redis (мгновенный отзыв при logout).
//!
//! Схема ключей: `blacklist:access:{jti}` = `"1"` c TTL = остаток жизни токена.
//! По истечении TTL ключ исчезает сам — токен к этому моменту уже невалиден.

use std::time::Duration;

use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use uuid::Uuid;

use crate::application::ports::{AccessTokenBlacklist, BlacklistError};

#[derive(Clone)]
pub struct RedisAccessTokenBlacklist {
    manager: ConnectionManager,
}

impl RedisAccessTokenBlacklist {
    pub fn new(manager: ConnectionManager) -> Self {
        Self { manager }
    }

    fn key(jti: Uuid) -> String {
        format!("blacklist:access:{jti}")
    }
}

#[async_trait]
impl AccessTokenBlacklist for RedisAccessTokenBlacklist {
    async fn revoke(&self, jti: Uuid, ttl: Duration) -> Result<(), BlacklistError> {
        let secs = ttl.as_secs();
        if secs == 0 {
            return Ok(());
        }

        let mut conn = self.manager.clone();
        let _: () = conn
            .set_ex(Self::key(jti), 1u8, secs)
            .await
            .map_err(|e| BlacklistError::Backend(e.into()))?;
        Ok(())
    }

    async fn is_revoked(&self, jti: Uuid) -> Result<bool, BlacklistError> {
        let mut conn = self.manager.clone();
        let revoked: bool = conn
            .exists(Self::key(jti))
            .await
            .map_err(|e| BlacklistError::Backend(e.into()))?;
        Ok(revoked)
    }
}
