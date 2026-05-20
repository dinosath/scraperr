use axum::{
    body::Body,
    extract::State,
    response::Response,
    Json,
};
use serde::Deserialize;

use crate::config::AppConfig;
use crate::error::AppError;
use scraperr_core::services::ai::{AiService, ChatMessage};

#[derive(Deserialize)]
pub struct AiChatRequest {
    pub messages: Vec<ChatMessage>,
}

pub async fn ai_chat(
    State(config): State<AppConfig>,
    Json(req): Json<AiChatRequest>,
) -> Result<Response, AppError> {
    let ai_config = config.ai_config();
    let response = AiService::openai_chat(&ai_config, &req.messages).await?;

    // Stream the OpenAI response through to the client
    let body = Body::from_stream(response.bytes_stream());

    Ok(Response::builder()
        .header("content-type", "text/plain")
        .body(body)
        .unwrap())
}

pub async fn check_ai(State(config): State<AppConfig>) -> Json<serde_json::Value> {
    let ai_config = config.ai_config();
    Json(serde_json::json!({
        "ai_enabled": AiService::check(&ai_config),
    }))
}
