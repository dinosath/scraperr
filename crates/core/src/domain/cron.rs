use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobInput {
    pub id: Option<String>,
    pub user_email: String,
    pub job_id: String,
    pub cron_expression: String,
}
