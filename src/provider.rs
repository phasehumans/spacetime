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

// ---------------------------------------------------------
// 1. OpenAI / OpenRouter / Custom Provider
// ---------------------------------------------------------
pub struct OpenAiProvider {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub client: reqwest::Client,
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
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
            return Err(anyhow!("OpenAI API error: {}", err_text));
        }

        let resp_json: serde_json::Value = resp.json().await?;
        let content = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid OpenAI response payload structure"))?;

        parse_agent_json(content)
    }
}

// ---------------------------------------------------------
// 2. Anthropic Provider
// ---------------------------------------------------------
pub struct AnthropicProvider {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub client: reqwest::Client,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat(&self, messages: &[Message]) -> Result<AgentResponse> {
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

// ---------------------------------------------------------
// 3. Gemini Provider
// ---------------------------------------------------------
pub struct GeminiProvider {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub client: reqwest::Client,
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn chat(&self, messages: &[Message]) -> Result<AgentResponse> {
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

// ---------------------------------------------------------
// 4. Ollama Provider
// ---------------------------------------------------------
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

// ---------------------------------------------------------
// Factory Function
// ---------------------------------------------------------
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_openai_provider_wiremock() {
        let mock_server = MockServer::start().await;

        let response_body = json!({
            "choices": [{
                "message": {
                    "content": "{\"reasoning\":\"Fix syntax in nginx.conf\",\"command\":\"nginx -t\"}"
                }
            }]
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let provider = OpenAiProvider {
            api_key: "test-key".to_string(),
            model: "gpt-4o".to_string(),
            base_url: mock_server.uri(),
            client: reqwest::Client::new(),
        };

        let messages = vec![Message {
            role: "user".to_string(),
            content: "Fix nginx config".to_string(),
        }];

        let res = provider.chat(&messages).await.unwrap();
        assert_eq!(res.reasoning, "Fix syntax in nginx.conf");
        assert_eq!(res.command, Some("nginx -t".to_string()));
    }

    #[tokio::test]
    async fn test_anthropic_provider_wiremock() {
        let mock_server = MockServer::start().await;

        let response_body = json!({
            "content": [{
                "text": "{\"reasoning\":\"Check port conflicts\",\"command\":\"netstat -tulpn\"}"
            }]
        });

        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(header("x-api-key", "anthropic-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let provider = AnthropicProvider {
            api_key: "anthropic-key".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: mock_server.uri(),
            client: reqwest::Client::new(),
        };

        let messages = vec![Message {
            role: "user".to_string(),
            content: "Resolve port conflict".to_string(),
        }];

        let res = provider.chat(&messages).await.unwrap();
        assert_eq!(res.reasoning, "Check port conflicts");
        assert_eq!(res.command, Some("netstat -tulpn".to_string()));
    }

    #[tokio::test]
    async fn test_gemini_provider_wiremock() {
        let mock_server = MockServer::start().await;

        let response_body = json!({
            "candidates": [{
                "content": {
                    "parts": [{
                        "text": "{\"reasoning\":\"Inspect logs\",\"command\":\"cat /var/log/nginx/error.log\"}"
                    }]
                }
            }]
        });

        Mock::given(method("POST"))
            .and(path("/v1beta/models/gemini-1.5-pro:generateContent"))
            .and(query_param("key", "gemini-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let provider = GeminiProvider {
            api_key: "gemini-key".to_string(),
            model: "gemini-1.5-pro".to_string(),
            base_url: mock_server.uri(),
            client: reqwest::Client::new(),
        };

        let messages = vec![Message {
            role: "user".to_string(),
            content: "Check logs".to_string(),
        }];

        let res = provider.chat(&messages).await.unwrap();
        assert_eq!(res.reasoning, "Inspect logs");
        assert_eq!(res.command, Some("cat /var/log/nginx/error.log".to_string()));
    }

    #[tokio::test]
    async fn test_ollama_provider_wiremock() {
        let mock_server = MockServer::start().await;

        let response_body = json!({
            "message": {
                "content": "{\"reasoning\":\"Create symlink\",\"command\":\"ln -s /a /b\"}"
            }
        });

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;

        let provider = OllamaProvider {
            model: "llama3".to_string(),
            base_url: mock_server.uri(),
            client: reqwest::Client::new(),
        };

        let messages = vec![Message {
            role: "user".to_string(),
            content: "Link files".to_string(),
        }];

        let res = provider.chat(&messages).await.unwrap();
        assert_eq!(res.reasoning, "Create symlink");
        assert_eq!(res.command, Some("ln -s /a /b".to_string()));
    }

    #[test]
    fn test_create_provider_factory() {
        let config = AppConfig {
            provider: "anthropic".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            api_key: Some("test-key".to_string()),
            base_url: None,
            max_turns: 15,
            timeout_seconds: 300,
        };

        let provider = create_provider(&config);
        assert!(provider.is_ok());
    }
}
