use chrono::Utc;
use sea_orm::{DatabaseConnection, Set};
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

use crate::domain::job::UpdatableJobField;
use scraperr_db::entities::job;
use scraperr_db::repositories::job::JobRepository;

#[derive(Debug, Error)]
pub enum JobError {
    #[error("Job not found: {0}")]
    NotFound(String),
    #[error("Forbidden")]
    Forbidden,
    #[error("Invalid field: {0}")]
    InvalidField(String),
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

pub struct JobService;

impl JobService {
    pub async fn submit(
        db: &DatabaseConnection,
        id: Option<String>,
        url: String,
        elements: serde_json::Value,
        user_email: &str,
        job_options: serde_json::Value,
        agent_mode: bool,
        prompt: Option<String>,
    ) -> Result<String, JobError> {
        let id = id.unwrap_or_else(|| Uuid::new_v4().simple().to_string());

        // Check if job already exists (re-submission)
        if let Some(_existing) = JobRepository::find_by_id(db, &id).await? {
            info!("Re-submitting existing job: {}", id);
            // Reset the job
            JobRepository::update_status(db, &[id.clone()], "Queued").await?;
            JobRepository::update_result(db, &id, serde_json::json!([])).await?;
            return Ok(id);
        }

        let now = Utc::now().fixed_offset();
        let model = job::ActiveModel {
            id: Set(id.clone()),
            url: Set(url),
            elements: Set(elements),
            user: Set(Some(user_email.to_string())),
            time_created: Set(now),
            result: Set(serde_json::json!([])),
            status: Set("Queued".to_string()),
            chat: Set(None),
            job_options: Set(Some(job_options)),
            agent_mode: Set(agent_mode),
            prompt: Set(prompt),
            favorite: Set(false),
        };

        JobRepository::insert(db, model).await?;
        info!("Created job: {}", id);
        Ok(id)
    }

    pub async fn get_by_id(
        db: &DatabaseConnection,
        id: &str,
        user_email: &str,
    ) -> Result<job::Model, JobError> {
        let job = JobRepository::find_by_id(db, id)
            .await?
            .ok_or_else(|| JobError::NotFound(id.to_string()))?;

        if job.user.as_deref() != Some(user_email) {
            return Err(JobError::Forbidden);
        }

        Ok(job)
    }

    pub async fn list_for_user(
        db: &DatabaseConnection,
        user_email: &str,
    ) -> Result<Vec<job::Model>, JobError> {
        Ok(JobRepository::find_by_user(db, user_email).await?)
    }

    pub async fn update(
        db: &DatabaseConnection,
        ids: &[String],
        field: &str,
        value: serde_json::Value,
    ) -> Result<(), JobError> {
        let field = field
            .parse::<UpdatableJobField>()
            .map_err(|e| JobError::InvalidField(e))?;

        match field {
            UpdatableJobField::Status => {
                let status = value.as_str().unwrap_or("Unknown");
                JobRepository::update_status(db, ids, status).await?;
            }
            UpdatableJobField::Favorite => {
                let fav = value.as_bool().unwrap_or(false);
                JobRepository::update_favorite(db, ids, fav).await?;
            }
            UpdatableJobField::Chat => {
                for id in ids {
                    JobRepository::update_chat(db, id, value.clone()).await?;
                }
            }
            UpdatableJobField::Result => {
                for id in ids {
                    JobRepository::update_result(db, id, value.clone()).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn delete(db: &DatabaseConnection, ids: &[String]) -> Result<(), JobError> {
        JobRepository::delete_by_ids(db, ids).await?;
        Ok(())
    }
}
