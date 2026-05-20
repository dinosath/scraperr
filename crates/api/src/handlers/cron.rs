use axum::{extract::State, Json};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::error::AppError;
use crate::middleware::oidc::AuthenticatedUser;
use scraperr_core::domain::cron::CronJobInput;
use scraperr_core::services::cron::CronService;

#[derive(Deserialize)]
pub struct ScheduleCronJobRequest {
    pub id: Option<String>,
    pub user_email: String,
    pub job_id: String,
    pub cron_expression: String,
}

pub async fn schedule_cron_job(
    State(db): State<DatabaseConnection>,
    _user: AuthenticatedUser,
    Json(req): Json<ScheduleCronJobRequest>,
) -> Result<Json<scraperr_db::entities::cron_job::Model>, AppError> {
    let cron = CronService::schedule(
        &db,
        CronJobInput {
            id: req.id,
            user_email: req.user_email,
            job_id: req.job_id,
            cron_expression: req.cron_expression,
        },
    )
    .await?;

    Ok(Json(cron))
}

pub async fn retrieve_cron_jobs(
    State(db): State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<scraperr_db::entities::cron_job::Model>>, AppError> {
    let jobs = CronService::list_for_user(&db, &user.email).await?;
    Ok(Json(jobs))
}

#[derive(Deserialize)]
pub struct DeleteCronJobRequest {
    pub id: String,
    pub user_email: String,
}

pub async fn delete_cron_job(
    State(db): State<DatabaseConnection>,
    _user: AuthenticatedUser,
    Json(req): Json<DeleteCronJobRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    CronService::delete(&db, &req.id, &req.user_email).await?;
    Ok(Json(serde_json::json!({ "message": "Cron job deleted." })))
}
