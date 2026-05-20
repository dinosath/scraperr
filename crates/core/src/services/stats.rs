use sea_orm::{
    ColumnTrait, DatabaseConnection, DbErr, EntityTrait, FromQueryResult, QueryFilter,
};
use serde::Serialize;
use thiserror::Error;

use scraperr_db::entities::job;
use scraperr_db::entities::job::Entity as Job;

#[derive(Debug, Error)]
pub enum StatsError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct JobsPerDay {
    pub date: String,
    pub job_count: i64,
}

#[derive(Debug, Serialize, FromQueryResult)]
pub struct AvgElementsPerLink {
    pub date: String,
    pub average_elements: f64,
    pub count: i64,
}

pub struct StatsService;

impl StatsService {
    pub async fn get_jobs_per_day(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Vec<JobsPerDay>, StatsError> {
        // Using raw SQL for date aggregation since it varies by DB
        let results = Job::find()
            .filter(job::Column::User.eq(email))
            .all(db)
            .await?;

        // Group by date in Rust since date functions differ across SQLite/Postgres
        use std::collections::HashMap;
        let mut by_day: HashMap<String, i64> = HashMap::new();

        for j in &results {
            let date = j.time_created.format("%Y-%m-%d").to_string();
            *by_day.entry(date).or_insert(0) += 1;
        }

        let mut data: Vec<JobsPerDay> = by_day
            .into_iter()
            .map(|(date, job_count)| JobsPerDay { date, job_count })
            .collect();

        data.sort_by(|a, b| a.date.cmp(&b.date));
        Ok(data)
    }

    pub async fn get_avg_elements_per_link(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Vec<AvgElementsPerLink>, StatsError> {
        let results = Job::find()
            .filter(job::Column::User.eq(email))
            .all(db)
            .await?;

        use std::collections::HashMap;
        let mut by_day: HashMap<String, (f64, i64)> = HashMap::new();

        for j in &results {
            let date = j.time_created.format("%Y-%m-%d").to_string();
            let element_count = j
                .elements
                .as_array()
                .map(|a| a.len() as f64)
                .unwrap_or(0.0);
            let entry = by_day.entry(date).or_insert((0.0, 0));
            entry.0 += element_count;
            entry.1 += 1;
        }

        let mut data: Vec<AvgElementsPerLink> = by_day
            .into_iter()
            .map(|(date, (total, count))| AvgElementsPerLink {
                date,
                average_elements: if count > 0 {
                    total / count as f64
                } else {
                    0.0
                },
                count,
            })
            .collect();

        data.sort_by(|a, b| a.date.cmp(&b.date));
        Ok(data)
    }
}
