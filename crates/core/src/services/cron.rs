use chrono::Utc;
use sea_orm::{DatabaseConnection, Set};
use thiserror::Error;
use uuid::Uuid;

use scraperr_db::entities::cron_job;
use scraperr_db::repositories::cron_job::CronJobRepository;

use crate::domain::cron::CronJobInput;

#[derive(Debug, Error)]
pub enum CronError {
    #[error("Cron job not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

pub struct CronService;

impl CronService {
    pub async fn schedule(
        db: &DatabaseConnection,
        input: CronJobInput,
    ) -> Result<cron_job::Model, CronError> {
        let id = input.id.unwrap_or_else(|| Uuid::new_v4().simple().to_string());
        let now = Utc::now().fixed_offset();

        let model = cron_job::ActiveModel {
            id: Set(id),
            user_email: Set(input.user_email),
            job_id: Set(input.job_id),
            cron_expression: Set(input.cron_expression),
            time_created: Set(now),
            time_updated: Set(now),
        };

        Ok(CronJobRepository::insert(db, model).await?)
    }

    pub async fn list_for_user(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Vec<cron_job::Model>, CronError> {
        Ok(CronJobRepository::find_by_user(db, email).await?)
    }

    pub async fn list_all(
        db: &DatabaseConnection,
    ) -> Result<Vec<cron_job::Model>, CronError> {
        Ok(CronJobRepository::find_all(db).await?)
    }

    pub async fn delete(
        db: &DatabaseConnection,
        id: &str,
        user_email: &str,
    ) -> Result<(), CronError> {
        CronJobRepository::delete(db, id, user_email).await?;
        Ok(())
    }
}
