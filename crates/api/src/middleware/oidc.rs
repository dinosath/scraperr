use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

/// Configuration for OIDC token validation.
#[derive(Clone)]
pub struct OidcConfig {
    pub issuer: String,
    pub audience: String,
    pub jwks_uri: String,
    pub jwks: Arc<RwLock<Option<jsonwebtoken::jwk::JwkSet>>>,
}

/// OIDC JWT claims.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
    #[serde(rename = "preferred_username")]
    pub preferred_username: Option<String>,
    pub name: Option<String>,
    pub exp: usize,
    pub iss: String,
    #[serde(default)]
    pub aud: serde_json::Value,
}

/// Extracted authenticated user available in handlers.
#[derive(Debug, Clone, Serialize)]
pub struct AuthenticatedUser {
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
}

/// Fetch JWKS from the IdP and cache it.
pub async fn refresh_jwks(config: &OidcConfig) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let resp = client.get(&config.jwks_uri).send().await?;
    let jwks: jsonwebtoken::jwk::JwkSet = resp.json().await?;
    let mut lock = config.jwks.write().await;
    *lock = Some(jwks);
    debug!("JWKS refreshed from {}", config.jwks_uri);
    Ok(())
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract token from Authorization header
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| {
                warn!("Missing or invalid Authorization header");
                StatusCode::UNAUTHORIZED
            })?;

        // Get OIDC config from extensions
        let oidc = parts
            .extensions
            .get::<Arc<OidcConfig>>()
            .cloned()
            .ok_or_else(|| {
                error!("OidcConfig not found in request extensions");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let jwks_guard = oidc.jwks.read().await;
        let jwk_set = jwks_guard.as_ref().ok_or_else(|| {
            error!("JWKS not loaded");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Try to decode with each key until one works
        let mut last_err = None;
        for jwk in &jwk_set.keys {
            let decoding_key = match DecodingKey::from_jwk(jwk) {
                Ok(k) => k,
                Err(e) => {
                    last_err = Some(e);
                    continue;
                }
            };

            let alg = jwk
                .common
                .key_algorithm
                .and_then(|a| match a {
                    jsonwebtoken::jwk::KeyAlgorithm::RS256 => Some(Algorithm::RS256),
                    jsonwebtoken::jwk::KeyAlgorithm::RS384 => Some(Algorithm::RS384),
                    jsonwebtoken::jwk::KeyAlgorithm::RS512 => Some(Algorithm::RS512),
                    jsonwebtoken::jwk::KeyAlgorithm::ES256 => Some(Algorithm::ES256),
                    jsonwebtoken::jwk::KeyAlgorithm::ES384 => Some(Algorithm::ES384),
                    _ => None,
                })
                .unwrap_or(Algorithm::RS256);

            let mut validation = Validation::new(alg);
            validation.set_audience(&[&oidc.audience]);
            validation.set_issuer(&[&oidc.issuer]);

            match decode::<Claims>(token, &decoding_key, &validation) {
                Ok(token_data) => {
                    let claims = token_data.claims;
                    let email = claims
                        .email
                        .or(claims.preferred_username)
                        .unwrap_or(claims.sub.clone());

                    return Ok(AuthenticatedUser {
                        sub: claims.sub,
                        email,
                        name: claims.name,
                    });
                }
                Err(e) => {
                    last_err = Some(e);
                    continue;
                }
            }
        }

        if let Some(e) = last_err {
            warn!("All JWKS keys failed to validate token: {}", e);
        }
        Err(StatusCode::UNAUTHORIZED)
    }
}
