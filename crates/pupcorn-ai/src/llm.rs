use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{AiError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String>;
}

/// OpenAI-compatible client (works for OpenAI and Ollama via base_url).
pub struct OpenAiClient {
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    client: reqwest::Client,
}

impl OpenAiClient {
    pub fn new(base_url: String, api_key: Option<String>, model: String) -> Self {
        Self {
            base_url,
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }

    pub fn from_env() -> Self {
        if let Ok(key) = std::env::var("GROQ_API_KEY") {
            return Self::new(
                "https://api.groq.com/openai/v1".to_string(),
                Some(key),
                std::env::var("GROQ_MODEL").unwrap_or_else(|_| "llama-3.3-70b-versatile".to_string()),
            );
        }
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            let base_url = std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
            let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
            return Self::new(base_url, Some(key), model);
        }
        // Free no-key fallback (may be rate-limited).
        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://text.pollinations.ai/openai".to_string());
        let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "openai".to_string());
        Self::new(base_url, None, model)
    }
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}
#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}
#[derive(Deserialize)]
struct ChoiceMessage {
    content: String,
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
        });
        for attempt in 0..3 {
            let mut req = self.client.post(&url).json(&body);
            if let Some(k) = &self.api_key {
                req = req.bearer_auth(k);
            }
            match req.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().await.map_err(|e| AiError::Llm(e.to_string()))?;
                    if status.is_success() {
                        let parsed: ChatResponse =
                            serde_json::from_str(&text).map_err(|e| AiError::Parse(e.to_string()))?;
                        return Ok(parsed
                            .choices
                            .into_iter()
                            .next()
                            .map(|c| c.message.content)
                            .unwrap_or_default());
                    }
                    if status.as_u16() == 503 && attempt < 2 {
                        tokio::time::sleep(std::time::Duration::from_secs(2 + attempt as u64 * 3)).await;
                        continue;
                    }
                    return Err(AiError::Llm(format!("{}: {}", status, text)));
                }
                Err(e) => {
                    if attempt < 2 {
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                    return Err(AiError::Llm(e.to_string()));
                }
            }
        }
        Err(AiError::Llm("max retries exceeded".to_string()))
    }
}

/// Mock LLM for tests/offline.
pub struct MockLlm {
    pub response: String,
}
#[async_trait]
impl LlmClient for MockLlm {
    async fn chat(&self, _messages: Vec<ChatMessage>) -> Result<String> {
        Ok(self.response.clone())
    }
}