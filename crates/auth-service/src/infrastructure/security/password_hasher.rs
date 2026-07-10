//! Хеширование паролей алгоритмом Argon2id.
//!
//! Argon2 — CPU/память-затратная операция, поэтому выполняется на blocking-пуле
//! Tokio, чтобы не блокировать асинхронный исполнитель.

use argon2::Argon2;
use argon2::password_hash::{
    PasswordHash as PhcHash, PasswordHasher as _, PasswordVerifier as _, SaltString,
};
use async_trait::async_trait;
use rand_core::OsRng;
use tokio::task;

use crate::application::ports::{PasswordHasher, PasswordHasherError};
use crate::domain::PasswordHash;

/// Хешер на основе Argon2id с параметрами по умолчанию (рекомендованные OWASP).
#[derive(Clone, Default)]
pub struct Argon2PasswordHasher;

#[async_trait]
impl PasswordHasher for Argon2PasswordHasher {
    async fn hash(&self, plaintext: &str) -> Result<PasswordHash, PasswordHasherError> {
        let plaintext = plaintext.to_owned();

        let phc: Result<String, String> = task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            Argon2::default()
                .hash_password(plaintext.as_bytes(), &salt)
                .map(|h| h.to_string())
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| PasswordHasherError::Hash(anyhow::anyhow!("hashing task failed: {e}")))?;

        let phc = phc.map_err(|e| PasswordHasherError::Hash(anyhow::anyhow!(e)))?;
        PasswordHash::from_hash(phc).map_err(|e| PasswordHasherError::Hash(anyhow::anyhow!(e)))
    }

    async fn verify(
        &self,
        plaintext: &str,
        hash: &PasswordHash,
    ) -> Result<bool, PasswordHasherError> {
        let plaintext = plaintext.to_owned();
        let hash = hash.as_str().to_owned();

        let result: Result<bool, String> = task::spawn_blocking(move || {
            let parsed = PhcHash::new(&hash).map_err(|e| e.to_string())?;
            Ok(Argon2::default()
                .verify_password(plaintext.as_bytes(), &parsed)
                .is_ok())
        })
        .await
        .map_err(|e| PasswordHasherError::Verify(anyhow::anyhow!("verify task failed: {e}")))?;

        result.map_err(|e| PasswordHasherError::Verify(anyhow::anyhow!(e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hashes_and_verifies_password() {
        let hasher = Argon2PasswordHasher;
        let hash = hasher.hash("correct horse battery staple").await.unwrap();

        // Хеш не совпадает с открытым текстом и имеет PHC-формат Argon2id.
        assert!(hash.as_str().starts_with("$argon2id$"));

        assert!(
            hasher
                .verify("correct horse battery staple", &hash)
                .await
                .unwrap()
        );
        assert!(!hasher.verify("wrong password", &hash).await.unwrap());
    }

    #[tokio::test]
    async fn distinct_salts_produce_distinct_hashes() {
        let hasher = Argon2PasswordHasher;
        let a = hasher.hash("same-input").await.unwrap();
        let b = hasher.hash("same-input").await.unwrap();
        assert_ne!(a.as_str(), b.as_str());
    }
}
