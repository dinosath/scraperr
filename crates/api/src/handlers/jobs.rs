use axum::{
    body::Body,
    extract::{Path, State},
    http::header,
    response::Response,
    Json,
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::middleware::oidc::AuthenticatedUser;
use scraperr_core::services::jobs::JobService;

// --- Trigger Job (manual execution) ---

pub async fn trigger_job(
    State(db): State<DatabaseConnection>,
    _user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let db_clone = db.clone();
    tokio::spawn(async move {
        if let Err(e) = scraperr_worker::execute_job_by_id(&db_clone, &id).await {
            tracing::error!("Manual job trigger failed for {id}: {e}");
        }
    });
    Ok(Json(serde_json::json!({ "message": "Job triggered" })))
}

// --- Submit Job ---

#[derive(Deserialize)]
pub struct SubmitJobRequest {
    pub id: Option<String>,
    pub url: String,
    pub elements: serde_json::Value,
    pub job_options: serde_json::Value,
    #[serde(default)]
    pub agent_mode: bool,
    pub prompt: Option<String>,
}

#[derive(Serialize)]
pub struct SubmitJobResponse {
    pub id: String,
    pub message: String,
}

pub async fn submit_scrape_job(
    State(db): State<DatabaseConnection>,
    user: AuthenticatedUser,
    Json(req): Json<SubmitJobRequest>,
) -> Result<Json<SubmitJobResponse>, AppError> {
    let id = JobService::submit(
        &db,
        req.id,
        req.url,
        req.elements,
        &user.email,
        req.job_options,
        req.agent_mode,
        req.prompt,
    )
    .await?;

    Ok(Json(SubmitJobResponse {
        id,
        message: "Job submitted successfully.".to_string(),
    }))
}

// --- Retrieve Jobs ---

pub async fn retrieve_scrape_jobs(
    State(db): State<DatabaseConnection>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<scraperr_db::entities::job::Model>>, AppError> {
    let jobs = JobService::list_for_user(&db, &user.email).await?;
    Ok(Json(jobs))
}

// --- Get Single Job ---

pub async fn get_job(
    State(db): State<DatabaseConnection>,
    user: AuthenticatedUser,
    Path(id): Path<String>,
) -> Result<Json<scraperr_db::entities::job::Model>, AppError> {
    let job = JobService::get_by_id(&db, &id, &user.email).await?;
    Ok(Json(job))
}

// --- Update Jobs ---

#[derive(Deserialize)]
pub struct UpdateJobsRequest {
    pub ids: Vec<String>,
    pub field: String,
    pub value: serde_json::Value,
}

pub async fn update_jobs(
    State(db): State<DatabaseConnection>,
    _user: AuthenticatedUser,
    Json(req): Json<UpdateJobsRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    JobService::update(&db, &req.ids, &req.field, req.value).await?;
    Ok(Json(serde_json::json!({ "message": "Jobs updated successfully" })))
}

// --- Delete Jobs ---

#[derive(Deserialize)]
pub struct DeleteJobsRequest {
    pub ids: Vec<String>,
}

pub async fn delete_scrape_jobs(
    State(db): State<DatabaseConnection>,
    _user: AuthenticatedUser,
    Json(req): Json<DeleteJobsRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    JobService::delete(&db, &req.ids).await?;
    Ok(Json(serde_json::json!({ "message": "Jobs successfully deleted." })))
}

// --- Download Jobs ---

#[derive(Deserialize)]
pub struct DownloadJobRequest {
    pub ids: Vec<String>,
    pub job_format: String,
}

pub async fn download(
    State(db): State<DatabaseConnection>,
    _user: AuthenticatedUser,
    Json(req): Json<DownloadJobRequest>,
) -> Result<Response, AppError> {
    use scraperr_db::repositories::job::JobRepository;

    let jobs = JobRepository::find_by_ids(&db, &req.ids).await.map_err(AppError::Database)?;

    match req.job_format.as_str() {
        "csv" => {
            let mut csv_content = String::from("id,url,element_name,xpath,text,user,time_created\n");

            for job in &jobs {
                if let Some(results) = job.result.as_array() {
                    for result_item in results {
                        if let Some(obj) = result_item.as_object() {
                            for (_url, elements) in obj {
                                if let Some(el_obj) = elements.as_object() {
                                    for (element_name, values) in el_obj {
                                        if let Some(vals) = values.as_array() {
                                            for val in vals {
                                                let text = val
                                                    .get("text")
                                                    .and_then(|t| t.as_str())
                                                    .unwrap_or("")
                                                    .replace('"', "\"\"");
                                                let xpath = val
                                                    .get("xpath")
                                                    .and_then(|t| t.as_str())
                                                    .unwrap_or("");
                                                if !text.trim().is_empty() {
                                                    csv_content.push_str(&format!(
                                                        "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                                                        job.id,
                                                        job.url,
                                                        element_name,
                                                        xpath,
                                                        text,
                                                        job.user.as_deref().unwrap_or(""),
                                                        job.time_created,
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Ok(Response::builder()
                .header(header::CONTENT_TYPE, "text/csv")
                .header(
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=export.csv",
                )
                .body(Body::from(csv_content))
                .unwrap())
        }
        "md" => {
            let mut md = String::new();
            for job in &jobs {
                md.push_str(&format!("# Job: {}\n\n", job.id));
                md.push_str(&format!("**URL:** {}\n\n", job.url));
                md.push_str(&format!(
                    "**Results:**\n```json\n{}\n```\n\n",
                    serde_json::to_string_pretty(&job.result).unwrap_or_default()
                ));
            }

            Ok(Response::builder()
                .header(header::CONTENT_TYPE, "text/markdown")
                .header(
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=export.md",
                )
                .body(Body::from(md))
                .unwrap())
        }
        _ => Err(AppError::Validation(format!(
            "Unsupported format: {}",
            req.job_format
        ))),
    }
}
