use axum::{extract::State, Json};
use sea_orm::DatabaseConnection;

use crate::error::AppError;
use crate::middleware::oidc::AuthenticatedUser;
use scraperr_core::services::stats::StatsService;

pub async fn avg_elements_per_link(
    State(db): State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let data = StatsService::get_avg_elements_per_link(&db, &user.email).await?;
    Ok(Json(serde_json::to_value(data).unwrap_or_default()))
}

pub async fn avg_jobs_per_day(
    State(db): State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let data = StatsService::get_jobs_per_day(&db, &user.email).await?;
    Ok(Json(serde_json::to_value(data).unwrap_or_default()))
}
