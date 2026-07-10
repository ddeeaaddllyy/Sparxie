//! Выпуск и проверка JWT, подписанных EdDSA (Ed25519).
//!
//! Приватный ключ хранится только в auth-сервисе. Публичный ключ раздаётся
//! сервисам-клиентам, которые валидируют access-токены локально, без обращения
//! к auth-сервису.

use std::path::Path;
use std::time::Duration;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use time::{Duration as TimeDuration, OffsetDateTime};
use uuid::Uuid;

use crate::application::ports::{
    AccessClaims, IssuedToken, RefreshClaims, TokenError, TokenService,
};
use crate::domain::UserId;

/// Вид токена — кодируется в claim `token_type`, чтобы access-токен нельзя было
/// предъявить там, где ожидается refresh, и наоборот.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TokenType {
    Access,
    Refresh,
}

/// Полезная нагрузка JWT.
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    jti: Uuid,
    iss: String,
    iat: i64,
    exp: i64,
    token_type: TokenType,
}

/// Token service на EdDSA (Ed25519).
pub struct Ed25519TokenService {
    encoding: EncodingKey,
    decoding: DecodingKey,
    issuer: String,
    access_ttl: Duration,
    refresh_ttl: Duration,
    validation: Validation,
    header: Header,
}

impl Ed25519TokenService {
    /// Создаёт сервис из PEM-ключей (PKCS#8 private, SPKI public).
    pub fn new(
        private_pem: &[u8],
        public_pem: &[u8],
        issuer: String,
        access_ttl: Duration,
        refresh_ttl: Duration,
    ) -> anyhow::Result<Self> {
        let encoding = EncodingKey::from_ed_pem(private_pem)?;
        let decoding = DecodingKey::from_ed_pem(public_pem)?;

        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_issuer(&[issuer.clone()]);
        validation.validate_exp = true;
        validation.validate_aud = false;

        Ok(Self {
            encoding,
            decoding,
            issuer,
            access_ttl,
            refresh_ttl,
            validation,
            header: Header::new(Algorithm::EdDSA),
        })
    }

    /// Читает ключи из файлов и создаёт сервис.
    pub fn from_files(
        private_key_path: impl AsRef<Path>,
        public_key_path: impl AsRef<Path>,
        issuer: String,
        access_ttl: Duration,
        refresh_ttl: Duration,
    ) -> anyhow::Result<Self> {
        let private_pem = std::fs::read(private_key_path.as_ref()).map_err(|e| {
            anyhow::anyhow!(
                "failed to read JWT private key {}: {e}",
                private_key_path.as_ref().display()
            )
        })?;
        let public_pem = std::fs::read(public_key_path.as_ref()).map_err(|e| {
            anyhow::anyhow!(
                "failed to read JWT public key {}: {e}",
                public_key_path.as_ref().display()
            )
        })?;
        Self::new(&private_pem, &public_pem, issuer, access_ttl, refresh_ttl)
    }

    fn issue(
        &self,
        user_id: UserId,
        ttl: Duration,
        token_type: TokenType,
    ) -> Result<IssuedToken, TokenError> {
        let now = OffsetDateTime::now_utc();
        let expires_at = now + TimeDuration::seconds(ttl.as_secs() as i64);
        let jti = Uuid::new_v4();

        let claims = Claims {
            sub: user_id.as_uuid(),
            jti,
            iss: self.issuer.clone(),
            iat: now.unix_timestamp(),
            exp: expires_at.unix_timestamp(),
            token_type,
        };

        let token = encode(&self.header, &claims, &self.encoding)
            .map_err(|e| TokenError::Issue(anyhow::anyhow!(e)))?;

        Ok(IssuedToken {
            token,
            jti,
            expires_at,
        })
    }

    fn decode_typed(&self, token: &str, expected: TokenType) -> Result<Claims, TokenError> {
        let data =
            decode::<Claims>(token, &self.decoding, &self.validation).map_err(map_jwt_error)?;

        if data.claims.token_type != expected {
            return Err(TokenError::Invalid);
        }
        Ok(data.claims)
    }
}

fn map_jwt_error(err: jsonwebtoken::errors::Error) -> TokenError {
    use jsonwebtoken::errors::ErrorKind;
    match err.kind() {
        ErrorKind::ExpiredSignature => TokenError::Expired,
        _ => TokenError::Invalid,
    }
}

fn expires_at_from_unix(exp: i64) -> Result<OffsetDateTime, TokenError> {
    OffsetDateTime::from_unix_timestamp(exp).map_err(|_| TokenError::Invalid)
}

impl TokenService for Ed25519TokenService {
    fn issue_access(&self, user_id: UserId) -> Result<IssuedToken, TokenError> {
        self.issue(user_id, self.access_ttl, TokenType::Access)
    }

    fn issue_refresh(&self, user_id: UserId) -> Result<IssuedToken, TokenError> {
        self.issue(user_id, self.refresh_ttl, TokenType::Refresh)
    }

    fn verify_access(&self, token: &str) -> Result<AccessClaims, TokenError> {
        let claims = self.decode_typed(token, TokenType::Access)?;
        Ok(AccessClaims {
            user_id: claims.sub,
            jti: claims.jti,
            expires_at: expires_at_from_unix(claims.exp)?,
        })
    }

    fn verify_refresh(&self, token: &str) -> Result<RefreshClaims, TokenError> {
        let claims = self.decode_typed(token, TokenType::Refresh)?;
        Ok(RefreshClaims {
            user_id: claims.sub,
            jti: claims.jti,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Одноразовая тестовая пара ключей (не используется нигде в production).
    const TEST_PRIV: &str = "-----BEGIN PRIVATE KEY-----\nMC4CAQAwBQYDK2VwBCIEINlLCgpASwXRk9a7DL7RkzB+fRPVU1XhFTFEvVEQ7HhD\n-----END PRIVATE KEY-----\n";
    const TEST_PUB: &str = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAdpZ/SO2njwqNCO9MF64Q1KLNwdhJr0Ho4Fv9lNe6LRE=\n-----END PUBLIC KEY-----\n";

    fn service() -> Ed25519TokenService {
        Ed25519TokenService::new(
            TEST_PRIV.as_bytes(),
            TEST_PUB.as_bytes(),
            "nedovolen-test".to_string(),
            Duration::from_secs(900),
            Duration::from_secs(1_209_600),
        )
        .expect("valid test keys")
    }

    #[test]
    fn access_token_roundtrip() {
        let svc = service();
        let user = UserId::new();
        let issued = svc.issue_access(user).unwrap();
        let claims = svc.verify_access(&issued.token).unwrap();
        assert_eq!(claims.user_id, user.as_uuid());
        assert_eq!(claims.jti, issued.jti);
    }

    #[test]
    fn refresh_token_roundtrip() {
        let svc = service();
        let user = UserId::new();
        let issued = svc.issue_refresh(user).unwrap();
        let claims = svc.verify_refresh(&issued.token).unwrap();
        assert_eq!(claims.user_id, user.as_uuid());
        assert_eq!(claims.jti, issued.jti);
    }

    #[test]
    fn access_token_cannot_be_used_as_refresh() {
        let svc = service();
        let issued = svc.issue_access(UserId::new()).unwrap();
        assert!(matches!(
            svc.verify_refresh(&issued.token),
            Err(TokenError::Invalid)
        ));
    }

    #[test]
    fn tampered_token_is_rejected() {
        let svc = service();
        let mut token = svc.issue_access(UserId::new()).unwrap().token;
        token.push('x');
        assert!(svc.verify_access(&token).is_err());
    }

    /// Токен, выпущенный auth-сервисом, должен приниматься клиентским
    /// верификатором из `shared` (совместимость формата claims).
    #[test]
    fn issued_access_token_is_accepted_by_shared_verifier() {
        let svc = service();
        let user = UserId::new();
        let issued = svc.issue_access(user).unwrap();

        let verifier =
            shared::jwt::AccessTokenVerifier::from_public_pem(TEST_PUB.as_bytes(), "nedovolen-test")
                .unwrap();
        let verified = verifier.verify(&issued.token).unwrap();
        assert_eq!(verified.user_id, user.as_uuid());
        assert_eq!(verified.jti, issued.jti);

        // refresh-токен НЕ должен приниматься как access.
        let refresh = svc.issue_refresh(user).unwrap();
        assert!(matches!(
            verifier.verify(&refresh.token),
            Err(shared::jwt::JwtError::WrongType)
        ));
    }
}
