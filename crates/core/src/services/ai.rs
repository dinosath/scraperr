use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, info};

#[derive(Debug, Error)]
pub enum AiError {
    #[error("AI not configured")]
    NotConfigured,
    #[error("AI request failed: {0}")]
    RequestFailed(String),
}

#[derive(Debug, Clone)]
pub struct AiConfig {
    pub openai_key: Option<String>,
    pub openai_model: Option<String>,
    pub ollama_url: Option<String>,
    pub ollama_model: Option<String>,
}

impl AiConfig {
    pub fn is_enabled(&self) -> bool {
        self.openai_key.is_some() || self.ollama_url.is_some()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub struct AiService;

impl AiService {
    pub fn check(config: &AiConfig) -> bool {
        config.is_enabled()
    }

    /// Stream chat via OpenAI API.
    /// Returns the response body as a reqwest::Response for streaming.
    pub async fn openai_chat(
        config: &AiConfig,
        messages: &[ChatMessage],
    ) -> Result<reqwest::Response, AiError> {
        let key = config.openai_key.as_deref().ok_or(AiError::NotConfigured)?;
        let model = config
            .openai_model
            .as_deref()
            .unwrap_or("gpt-4.1-mini");

        let client = reqwest::Client::new();
        let resp = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(key)
            .json(&serde_json::json!({
                "model": model,
                "messages": messages,
                "stream": true,
            }))
            .send()
            .await
            .map_err(|e| AiError::RequestFailed(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("OpenAI error: {} - {}", status, body);
            return Err(AiError::RequestFailed(format!("{status}: {body}")));
        }

        // For streaming we'd return the response to be streamed out.
        // Re-do the request without consuming it:
        let resp = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(key)
            .json(&serde_json::json!({
                "model": model,
                "messages": messages,
                "stream": true,
            }))
            .send()
            .await
            .map_err(|e| AiError::RequestFailed(e.to_string()))?;

        Ok(resp)
    }

    /// Non-streaming chat for simple use cases.
    pub async fn chat_completion(
        config: &AiConfig,
        messages: &[ChatMessage],
    ) -> Result<String, AiError> {
        let key = config.openai_key.as_deref().ok_or(AiError::NotConfigured)?;
        let model = config
            .openai_model
            .as_deref()
            .unwrap_or("gpt-4.1-mini");

        let client = reqwest::Client::new();
        let resp: serde_json::Value = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(key)
            .json(&serde_json::json!({
                "model": model,
                "messages": messages,
            }))
            .send()
            .await
            .map_err(|e| AiError::RequestFailed(e.to_string()))?
            .json()
            .await
            .map_err(|e| AiError::RequestFailed(e.to_string()))?;

        let content = resp["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        info!("AI response length: {} chars", content.len());
        Ok(content)
    }
}
