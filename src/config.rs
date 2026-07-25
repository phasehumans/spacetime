use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_turns: usize,
    pub timeout_seconds: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            api_key: None,
            base_url: None,
            max_turns: 15,
            timeout_seconds: 300,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PartialConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub max_turns: Option<usize>,
    pub timeout_seconds: Option<u64>,
}

pub struct ConfigResolver;

impl ConfigResolver {
    pub fn resolve(
        cli_provider: Option<String>,
        cli_model: Option<String>,
        cli_api_key: Option<String>,
        project_config_path: Option<&Path>,
    ) -> Result<AppConfig> {
        let mut config = AppConfig::default();

        // 1. Read global config (~/.config/spacetime/config.toml) if present
        if let Some(user_dirs) = directories::ProjectDirs::from("com", "spacetime", "spacetime") {
            let global_path = user_dirs.config_dir().join("config.toml");
            if global_path.exists() {
                if let Ok(contents) = fs::read_to_string(global_path) {
                    if let Ok(file_cfg) = toml::from_str::<PartialConfig>(&contents) {
                        Self::merge(&mut config, file_cfg);
                    }
                }
            }
        }

        // 2. Read project config (spacetime.toml or explicit path) if present
        let project_path = project_config_path
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("spacetime.toml"));

        if project_path.exists() {
            if let Ok(contents) = fs::read_to_string(project_path) {
                if let Ok(file_cfg) = toml::from_str::<PartialConfig>(&contents) {
                    Self::merge(&mut config, file_cfg);
                }
            }
        }

        // 3. Read environment variables
        if let Ok(val) = env::var("SPACETIME_PROVIDER") {
            config.provider = val;
        }
        if let Ok(val) = env::var("SPACETIME_MODEL") {
            config.model = val;
        }

        // Resolve API keys based on provider
        let env_key = match config.provider.to_lowercase().as_str() {
            "openai" => env::var("OPENAI_API_KEY").ok(),
            "anthropic" => env::var("ANTHROPIC_API_KEY").ok(),
            "gemini" | "google" => env::var("GEMINI_API_KEY").ok(),
            "openrouter" => env::var("OPENROUTER_API_KEY").ok(),
            _ => env::var("LLM_API_KEY").ok(),
        };

        if let Some(key) = env_key {
            config.api_key = Some(key);
        }

        // 4. CLI flags override everything
        if let Some(p) = cli_provider {
            config.provider = p;
        }
        if let Some(m) = cli_model {
            config.model = m;
        }
        if let Some(k) = cli_api_key {
            config.api_key = Some(k);
        }

        Ok(config)
    }

    fn merge(base: &mut AppConfig, partial: PartialConfig) {
        if let Some(p) = partial.provider {
            base.provider = p;
        }
        if let Some(m) = partial.model {
            base.model = m;
        }
        if let Some(k) = partial.api_key {
            base.api_key = Some(k);
        }
        if let Some(u) = partial.base_url {
            base.base_url = Some(u);
        }
        if let Some(t) = partial.max_turns {
            base.max_turns = t;
        }
        if let Some(sec) = partial.timeout_seconds {
            base.timeout_seconds = sec;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_resolve_default_config() {
        let config = ConfigResolver::resolve(None, None, None, None).unwrap();
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4o");
    }

    #[test]
    fn test_resolve_cli_overrides_toml() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
provider = "gemini"
model = "gemini-1.5-pro"
max_turns = 10
"#
        )
        .unwrap();

        let config = ConfigResolver::resolve(
            Some("anthropic".to_string()),
            Some("claude-3-5-sonnet-20241022".to_string()),
            None,
            Some(file.path()),
        )
        .unwrap();

        // CLI flags override file config
        assert_eq!(config.provider, "anthropic");
        assert_eq!(config.model, "claude-3-5-sonnet-20241022");
        // Non-overridden options come from file config
        assert_eq!(config.max_turns, 10);
    }
}
