use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized(String),
    Forbidden,
    Validation(String),
    Internal(String),
    Database(sea_orm::DbErr),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::Database(e) => {
                tracing::error!("Database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(e: sea_orm::DbErr) -> Self {
        AppError::Database(e)
    }
}

impl From<scraperr_core::services::jobs::JobError> for AppError {
    fn from(e: scraperr_core::services::jobs::JobError) -> Self {
        match e {
            scraperr_core::services::jobs::JobError::NotFound(msg) => AppError::NotFound(msg),
            scraperr_core::services::jobs::JobError::Forbidden => AppError::Forbidden,
            scraperr_core::services::jobs::JobError::InvalidField(msg) => {
                AppError::Validation(msg)
            }
            scraperr_core::services::jobs::JobError::Database(e) => AppError::Database(e),
        }
    }
}

impl From<scraperr_core::services::cron::CronError> for AppError {
    fn from(e: scraperr_core::services::cron::CronError) -> Self {
        match e {
            scraperr_core::services::cron::CronError::NotFound(msg) => AppError::NotFound(msg),
            scraperr_core::services::cron::CronError::Database(e) => AppError::Database(e),
        }
    }
}

impl From<scraperr_core::services::stats::StatsError> for AppError {
    fn from(e: scraperr_core::services::stats::StatsError) -> Self {
        match e {
            scraperr_core::services::stats::StatsError::Database(e) => AppError::Database(e),
        }
    }
}

impl From<scraperr_core::services::ai::AiError> for AppError {
    fn from(e: scraperr_core::services::ai::AiError) -> Self {
        match e {
            scraperr_core::services::ai::AiError::NotConfigured => {
                AppError::Internal("AI not configured".to_string())
            }
            scraperr_core::services::ai::AiError::RequestFailed(msg) => AppError::Internal(msg),
        }
    }
}
