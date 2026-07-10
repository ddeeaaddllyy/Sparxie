//! Оффлайн-верификация access-токенов для сервисов-клиентов.
//!
//! Клиенты (`requiem-service`, `zenith-service`) валидируют токены **локально**
//! по публичному ключу Ed25519, без сетевого вызова к auth-сервису. Формат
//! claims должен совпадать с тем, что выпускает auth-сервис.

use std::path::Path;

use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::Deserialize;
use uuid::Uuid;

/// Ошибка верификации токена.
#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("token is invalid")]
    Invalid,
    #[error("token is expired")]
    Expired,
    #[error("wrong token type")]
    WrongType,
}

/// Вид токена (совпадает с моделью auth-сервиса).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: Uuid,
    jti: Uuid,
    token_type: TokenType,
}

/// Данные проверенного access-токена.
#[derive(Debug, Clone)]
pub struct VerifiedAccess {
    pub user_id: Uuid,
    pub jti: Uuid,
}

/// Верификатор access-токенов по публичному ключу.
pub struct AccessTokenVerifier {
    decoding: DecodingKey,
    validation: Validation,
}

impl AccessTokenVerifier {
    /// Создаёт верификатор из публичного ключа (SPKI PEM) и ожидаемого issuer.
    pub fn from_public_pem(public_pem: &[u8], issuer: &str) -> anyhow::Result<Self> {
        let decoding = DecodingKey::from_ed_pem(public_pem)?;
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[issuer]);
        validation.validate_exp = true;
        validation.validate_aud = false;
        Ok(Self {
            decoding,
            validation,
        })
    }

    /// Читает публичный ключ из файла.
    pub fn from_file(public_key_path: impl AsRef<Path>, issuer: &str) -> anyhow::Result<Self> {
        let pem = std::fs::read(public_key_path.as_ref()).map_err(|e| {
            anyhow::anyhow!(
                "failed to read public key {}: {e}",
                public_key_path.as_ref().display()
            )
        })?;
        Self::from_public_pem(&pem, issuer)
    }

    /// Проверяет подпись/срок/issuer и что это именно access-токен.
    pub fn verify(&self, token: &str) -> Result<VerifiedAccess, JwtError> {
        let data = decode::<Claims>(token, &self.decoding, &self.validation).map_err(|e| {
            use jsonwebtoken::errors::ErrorKind;
            match e.kind() {
                ErrorKind::ExpiredSignature => JwtError::Expired,
                _ => JwtError::Invalid,
            }
        })?;

        if data.claims.token_type != TokenType::Access {
            return Err(JwtError::WrongType);
        }

        Ok(VerifiedAccess {
            user_id: data.claims.sub,
            jti: data.claims.jti,
        })
    }
}
