use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub grpc_port: u16,
    pub oidc_issuer: String,
    pub oidc_audience: String,
    pub oidc_jwks_uri: String,
    pub registration_enabled: bool,
    pub recordings_enabled: bool,
    pub openai_key: Option<String>,
    pub openai_model: Option<String>,
    pub ollama_url: Option<String>,
    pub ollama_model: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://data/database.db?mode=rwc".to_string()),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()?,
            grpc_port: env::var("GRPC_PORT")
                .unwrap_or_else(|_| "50051".to_string())
                .parse()?,
            oidc_issuer: env::var("OIDC_ISSUER")
                .unwrap_or_else(|_| "http://localhost:8080/realms/scraperr".to_string()),
            oidc_audience: env::var("OIDC_AUDIENCE")
                .unwrap_or_else(|_| "scraperr-api".to_string()),
            oidc_jwks_uri: env::var("OIDC_JWKS_URI").unwrap_or_else(|_| {
                "http://localhost:8080/realms/scraperr/protocol/openid-connect/certs".to_string()
            }),
            registration_enabled: env::var("REGISTRATION_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            recordings_enabled: env::var("RECORDINGS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            openai_key: env::var("OPENAI_KEY").ok(),
            openai_model: env::var("OPENAI_MODEL").ok(),
            ollama_url: env::var("OLLAMA_URL").ok(),
            ollama_model: env::var("OLLAMA_MODEL").ok(),
        })
    }

    pub fn ai_config(&self) -> scraperr_core::services::ai::AiConfig {
        scraperr_core::services::ai::AiConfig {
            openai_key: self.openai_key.clone(),
            openai_model: self.openai_model.clone(),
            ollama_url: self.ollama_url.clone(),
            ollama_model: self.ollama_model.clone(),
        }
    }
}
