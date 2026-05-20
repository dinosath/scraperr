use axum::{extract::State, Json};
use serde::Serialize;

use crate::config::AppConfig;
use crate::middleware::oidc::AuthenticatedUser;

#[derive(Serialize)]
pub struct AuthCheckResponse {
    pub registration: bool,
    pub recordings_enabled: bool,
}

pub async fn check_auth(State(config): State<AppConfig>) -> Json<AuthCheckResponse> {
    Json(AuthCheckResponse {
        registration: config.registration_enabled,
        recordings_enabled: config.recordings_enabled,
    })
}

#[derive(Serialize)]
pub struct UserResponse {
    pub email: String,
    pub full_name: Option<String>,
}

pub async fn get_me(user: AuthenticatedUser) -> Json<UserResponse> {
    Json(UserResponse {
        email: user.email,
        full_name: user.name,
    })
}
