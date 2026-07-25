use crate::config::AppConfig;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;

const SYSTEM_PROMPT: &str = r#"You are an autonomous AI agent in a Linux sandbox solving system challenges.
Your responses MUST be formatted in valid JSON with two keys:
{
  "reasoning": "Step-by-step reasoning explaining your plan",
  "command": "The exact bash command to execute in the container (or null if finished)"
}"#;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentResponse {
    pub reasoning: String,
    pub command: Option<String>,
}

pub fn parse_agent_json(text: &str) -> Result<AgentResponse> {
    let cleaned = text.trim();
    let json_str = if cleaned.contains("```json") {
        cleaned
            .split("```json")
            .nth(1)
            .unwrap_or(cleaned)
            .split("```")
            .next()
            .unwrap_or(cleaned)
            .trim()
    } else if cleaned.contains("```") {
        cleaned
            .split("```")
            .nth(1)
            .unwrap_or(cleaned)
            .split("```")
            .next()
            .unwrap_or(cleaned)
            .trim()
    } else {
        cleaned
    };

    if let Ok(parsed) = serde_json::from_str::<AgentResponse>(json_str) {
        return Ok(parsed);
    }

    Ok(AgentResponse {
        reasoning: text.to_string(),
        command: None,
    })
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, messages: &[Message]) -> Result<AgentResponse>;
}

pub struct OpenAiProvider {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub client: reqwest::Client,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn chat(&self, messages: &[Message]) -> Result<AgentResponse> {
        if self.api_key.is_empty() && !self.base_url.contains("localhost") && !self.base_url.contains("127.0.0.1") {
            return Err(anyhow!("No API key provided for OpenAI. Please export OPENAI_API_KEY=\"your_key\" or run with '--provider ollama'."));
        }

        let mut api_messages = vec![json!({
            "role": "system",
            "content": SYSTEM_PROMPT
        })];

        for msg in messages {
            api_messages.push(json!({
                "role": msg.role,
                "content": msg.content
            }));
        }

        let url = if self.base_url.ends_with("/chat/completions") {
            self.base_url.clone()
        } else {
            format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
        };

        let body = json!({
            "model": self.model,
            "messages": api_messages,
            "response_format": { "type": "json_object" }
        });

        let mut req = self.client.post(&url).json(&body);
        if !self.api_key.is_empty() {
            req = req.bearer_auth(&self.api_key);
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let err_text = resp.text().await?;
            if err_text.contains("invalid_api_key") || err_text.contains("You didn't provide an API key") {
                return Err(anyhow!("OpenAI API key missing or invalid. Please export OPENAI_API_KEY=\"your_key\"."));
            }
            return Err(anyhow!("OpenAI API error: {}", err_text));
        }

        let resp_json: serde_json::Value = resp.json().await?;
        let content = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid OpenAI response payload structure"))?;

        parse_agent_json(content)
    }
}

pub struct AnthropicProvider {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub client: reqwest::Client,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat(&self, messages: &[Message]) -> Result<AgentResponse> {
        if self.api_key.is_empty() {
            return Err(anyhow!("No API key provided for Anthropic. Please export ANTHROPIC_API_KEY=\"your_key\"."));
        }

        let mut api_messages = Vec::new();
        for msg in messages {
            if msg.role != "system" {
                api_messages.push(json!({
                    "role": msg.role,
                    "content": msg.content
                }));
            }
        }

        let url = if self.base_url.ends_with("/v1/messages") {
            self.base_url.clone()
        } else {
            format!("{}/v1/messages", self.base_url.trim_end_matches('/'))
        };

        let body = json!({
            "model": self.model,
            "system": SYSTEM_PROMPT,
            "messages": api_messages,
            "max_tokens": 1024
        });

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await?;
            return Err(anyhow!("Anthropic API error: {}", err_text));
        }

        let resp_json: serde_json::Value = resp.json().await?;
        let content = resp_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid Anthropic response structure"))?;

        parse_agent_json(content)
    }
}

pub struct GeminiProvider {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub client: reqwest::Client,
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn chat(&self, messages: &[Message]) -> Result<AgentResponse> {
        if self.api_key.is_empty() {
            return Err(anyhow!("No API key provided for Gemini. Please export GEMINI_API_KEY=\"your_key\"."));
        }

        let mut contents = Vec::new();
        for msg in messages {
            let role = if msg.role == "assistant" { "model" } else { "user" };
            contents.push(json!({
                "role": role,
                "parts": [{ "text": msg.content }]
            }));
        }

        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url.trim_end_matches('/'),
            self.model,
            self.api_key
        );

        let body = json!({
            "contents": contents,
            "systemInstruction": {
                "parts": [{ "text": SYSTEM_PROMPT }]
            }
        });

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await?;
            return Err(anyhow!("Gemini API error: {}", err_text));
        }

        let resp_json: serde_json::Value = resp.json().await?;
        let content = resp_json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid Gemini response structure"))?;

        parse_agent_json(content)
    }
}

pub struct OllamaProvider {
    pub model: String,
    pub base_url: String,
    pub client: reqwest::Client,
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn chat(&self, messages: &[Message]) -> Result<AgentResponse> {
        let mut api_messages = vec![json!({
            "role": "system",
            "content": SYSTEM_PROMPT
        })];

        for msg in messages {
            api_messages.push(json!({
                "role": msg.role,
                "content": msg.content
            }));
        }

        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

        let body = json!({
            "model": self.model,
            "messages": api_messages,
            "stream": false
        });

        let resp = self.client.post(&url).json(&body).send().await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await?;
            return Err(anyhow!("Ollama API error: {}", err_text));
        }

        let resp_json: serde_json::Value = resp.json().await?;
        let content = resp_json["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid Ollama response structure"))?;

        parse_agent_json(content)
    }
}

pub fn create_provider(config: &AppConfig) -> Result<Box<dyn LlmProvider>> {
    let client = reqwest::Client::new();
    let provider_name = config.provider.to_lowercase();
    let api_key = config.api_key.clone().unwrap_or_default();

    match provider_name.as_str() {
        "openai" => Ok(Box::new(OpenAiProvider {
            api_key,
            model: config.model.clone(),
            base_url: config.base_url.clone().unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            client,
        })),
        "anthropic" => Ok(Box::new(AnthropicProvider {
            api_key,
            model: config.model.clone(),
            base_url: config.base_url.clone().unwrap_or_else(|| "https://api.anthropic.com".to_string()),
            client,
        })),
        "gemini" | "google" => Ok(Box::new(GeminiProvider {
            api_key,
            model: config.model.clone(),
            base_url: config.base_url.clone().unwrap_or_else(|| "https://generativelanguage.googleapis.com".to_string()),
            client,
        })),
        "ollama" => Ok(Box::new(OllamaProvider {
            model: config.model.clone(),
            base_url: config.base_url.clone().unwrap_or_else(|| "http://localhost:11434".to_string()),
            client,
        })),
        "openrouter" => Ok(Box::new(OpenAiProvider {
            api_key,
            model: config.model.clone(),
            base_url: config.base_url.clone().unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string()),
            client,
        })),
        _ => Ok(Box::new(OpenAiProvider {
            api_key,
            model: config.model.clone(),
            base_url: config.base_url.clone().unwrap_or_else(|| "http://localhost:8000/v1".to_string()),
            client,
        })),
    }
}
