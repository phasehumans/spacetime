use serde_json::json;
use spacetime_cli::config::AppConfig;
use spacetime_cli::provider::{
    create_provider, AnthropicProvider, GeminiProvider, LlmProvider, Message, OllamaProvider,
    OpenAiProvider,
};
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
