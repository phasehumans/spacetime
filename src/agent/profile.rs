use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessType {
    ClaudeCode,
    GeminiCli,
    Antigravity,
    Codex,
    Aider,
    Devin,
    December,
    Pi,
    CursorCli,
    SweAgent,
    OpenHands,
    Goose,
    Plandex,
    Cline,
    Smolagents,
    Mentat,
    Custom,
}

impl fmt::Display for HarnessType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HarnessType::ClaudeCode => write!(f, "Claude Code"),
            HarnessType::GeminiCli => write!(f, "Gemini CLI"),
            HarnessType::Antigravity => write!(f, "Antigravity (AGY)"),
            HarnessType::Codex => write!(f, "OpenAI Codex"),
            HarnessType::Aider => write!(f, "Aider"),
            HarnessType::Devin => write!(f, "Devin"),
            HarnessType::December => write!(f, "December"),
            HarnessType::Pi => write!(f, "Pi"),
            HarnessType::CursorCli => write!(f, "Cursor CLI"),
            HarnessType::SweAgent => write!(f, "SWE-agent"),
            HarnessType::OpenHands => write!(f, "OpenHands"),
            HarnessType::Goose => write!(f, "Goose"),
            HarnessType::Plandex => write!(f, "Plandex"),
            HarnessType::Cline => write!(f, "Cline"),
            HarnessType::Smolagents => write!(f, "Smolagents"),
            HarnessType::Mentat => write!(f, "Mentat"),
            HarnessType::Custom => write!(f, "Custom CLI"),
        }
    }
}

impl HarnessType {
    pub fn description(&self) -> &'static str {
        match self {
            HarnessType::ClaudeCode => "anthropic official cli agent with tool calling",
            HarnessType::GeminiCli => "google deepmind gemini cli runner",
            HarnessType::Antigravity => "google deepmind antigravity agent cli",
            HarnessType::Codex => "openai codex autonomous terminal coding agent",
            HarnessType::Aider => "git-native pairing & file editing harness",
            HarnessType::Devin => "cognition ai autonomous software engineer cli",
            HarnessType::December => "december autonomous tui coding agent",
            HarnessType::Pi => "inflection ai personal terminal companion",
            HarnessType::CursorCli => "headless cursor ai coding agent",
            HarnessType::SweAgent => "princeton swe-agent lm terminal environment",
            HarnessType::OpenHands => "all-hands ai autonomous software developer",
            HarnessType::Goose => "block's open-source extensible ai developer agent",
            HarnessType::Plandex => "terminal ai engine for complex multi-file tasks",
            HarnessType::Cline => "roo-code / cline autonomous coding agent",
            HarnessType::Smolagents => "huggingface lightweight multi-modal code agent",
            HarnessType::Mentat => "interactive command-line coding assistant",
            HarnessType::Custom => "provide custom in-container executable & arguments",
        }
    }

    pub fn default_models(&self) -> Vec<ModelOption> {
        match self {
            HarnessType::ClaudeCode => vec![
                ModelOption {
                    id: "claude-3-7-sonnet-20250219".to_string(),
                    tag: "[thinking]".to_string(),
                    recommended: true,
                    note: "Hybrid reasoning & coding".to_string(),
                },
                ModelOption {
                    id: "claude-3-5-sonnet-20241022".to_string(),
                    tag: "[standard]".to_string(),
                    recommended: false,
                    note: "Fast and reliable".to_string(),
                },
                ModelOption {
                    id: "claude-3-5-haiku-20241022".to_string(),
                    tag: "[fast]".to_string(),
                    recommended: false,
                    note: "Lightweight & low latency".to_string(),
                },
            ],
            HarnessType::GeminiCli => vec![
                ModelOption {
                    id: "gemini-2.5-pro".to_string(),
                    tag: "[reasoning]".to_string(),
                    recommended: true,
                    note: "Top complex problem solver".to_string(),
                },
                ModelOption {
                    id: "gemini-2.5-flash".to_string(),
                    tag: "[ultra-fast]".to_string(),
                    recommended: false,
                    note: "Fast multimodal execution".to_string(),
                },
                ModelOption {
                    id: "gemini-2.0-flash".to_string(),
                    tag: "[standard]".to_string(),
                    recommended: false,
                    note: "General coding tasks".to_string(),
                },
            ],
            HarnessType::Antigravity => vec![
                ModelOption {
                    id: "gemini-2.5-pro".to_string(),
                    tag: "[reasoning]".to_string(),
                    recommended: true,
                    note: "DeepMind AGY 2.0 reasoning".to_string(),
                },
                ModelOption {
                    id: "gemini-2.5-flash".to_string(),
                    tag: "[fast]".to_string(),
                    recommended: false,
                    note: "Fast AGY tool calling".to_string(),
                },
                ModelOption {
                    id: "claude-3-7-sonnet".to_string(),
                    tag: "[sonnet]".to_string(),
                    recommended: false,
                    note: "Anthropic Claude on AGY".to_string(),
                },
            ],
            HarnessType::Codex => vec![
                ModelOption {
                    id: "gpt-4o".to_string(),
                    tag: "[flagship]".to_string(),
                    recommended: true,
                    note: "OpenAI flagship code model".to_string(),
                },
                ModelOption {
                    id: "o3-mini".to_string(),
                    tag: "[reasoning]".to_string(),
                    recommended: false,
                    note: "High-speed reasoning model".to_string(),
                },
                ModelOption {
                    id: "o1".to_string(),
                    tag: "[deep-reasoning]".to_string(),
                    recommended: false,
                    note: "Deep reasoning model".to_string(),
                },
            ],
            HarnessType::Aider => vec![
                ModelOption {
                    id: "claude-3-7-sonnet".to_string(),
                    tag: "[sonnet]".to_string(),
                    recommended: true,
                    note: "Default Aider architect".to_string(),
                },
                ModelOption {
                    id: "gemini/gemini-2.5-pro".to_string(),
                    tag: "[gemini]".to_string(),
                    recommended: false,
                    note: "Deep Google context".to_string(),
                },
                ModelOption {
                    id: "gpt-4o".to_string(),
                    tag: "[openai]".to_string(),
                    recommended: false,
                    note: "OpenAI flagship model".to_string(),
                },
                ModelOption {
                    id: "deepseek/deepseek-r1".to_string(),
                    tag: "[r1]".to_string(),
                    recommended: false,
                    note: "Open-weights reasoning".to_string(),
                },
            ],
            HarnessType::Devin => vec![
                ModelOption {
                    id: "devin-v1".to_string(),
                    tag: "[cognition]".to_string(),
                    recommended: true,
                    note: "Devin primary autonomous agent".to_string(),
                },
                ModelOption {
                    id: "devin-deep-reasoning".to_string(),
                    tag: "[reasoning]".to_string(),
                    recommended: false,
                    note: "Deep planning and verification".to_string(),
                },
            ],
            HarnessType::December => vec![
                ModelOption {
                    id: "claude-3-7-sonnet".to_string(),
                    tag: "[sonnet]".to_string(),
                    recommended: true,
                    note: "December default engine".to_string(),
                },
                ModelOption {
                    id: "gemini-2.5-pro".to_string(),
                    tag: "[gemini]".to_string(),
                    recommended: false,
                    note: "Google multimodal engine".to_string(),
                },
                ModelOption {
                    id: "gpt-4o".to_string(),
                    tag: "[openai]".to_string(),
                    recommended: false,
                    note: "OpenAI tool calling".to_string(),
                },
            ],
            HarnessType::Pi => vec![
                ModelOption {
                    id: "inflection-2.5".to_string(),
                    tag: "[inflection]".to_string(),
                    recommended: true,
                    note: "Inflection conversational engine".to_string(),
                },
                ModelOption {
                    id: "pi-fast".to_string(),
                    tag: "[fast]".to_string(),
                    recommended: false,
                    note: "Low latency quick responses".to_string(),
                },
            ],
            HarnessType::CursorCli => vec![
                ModelOption {
                    id: "claude-3-7-sonnet".to_string(),
                    tag: "[sonnet]".to_string(),
                    recommended: true,
                    note: "Cursor flagship model".to_string(),
                },
                ModelOption {
                    id: "cursor-small".to_string(),
                    tag: "[fast]".to_string(),
                    recommended: false,
                    note: "High speed code editing".to_string(),
                },
                ModelOption {
                    id: "gpt-4o".to_string(),
                    tag: "[openai]".to_string(),
                    recommended: false,
                    note: "OpenAI flagship".to_string(),
                },
            ],
            HarnessType::SweAgent => vec![
                ModelOption {
                    id: "claude-3-7-sonnet-20250219".to_string(),
                    tag: "[sonnet]".to_string(),
                    recommended: true,
                    note: "State-of-the-art SWE benchmark model".to_string(),
                },
                ModelOption {
                    id: "gpt-4o".to_string(),
                    tag: "[gpt-4o]".to_string(),
                    recommended: false,
                    note: "OpenAI tool calling agent".to_string(),
                },
                ModelOption {
                    id: "deepseek-reasoner".to_string(),
                    tag: "[r1]".to_string(),
                    recommended: false,
                    note: "DeepSeek R1 reasoning".to_string(),
                },
            ],
            HarnessType::OpenHands => vec![
                ModelOption {
                    id: "anthropic/claude-3-7-sonnet".to_string(),
                    tag: "[sonnet]".to_string(),
                    recommended: true,
                    note: "Primary agent model".to_string(),
                },
                ModelOption {
                    id: "openai/gpt-4o".to_string(),
                    tag: "[gpt-4o]".to_string(),
                    recommended: false,
                    note: "OpenAI tool calling".to_string(),
                },
                ModelOption {
                    id: "deepseek/deepseek-r1".to_string(),
                    tag: "[r1]".to_string(),
                    recommended: false,
                    note: "DeepSeek R1 model".to_string(),
                },
            ],
            HarnessType::Goose => vec![
                ModelOption {
                    id: "claude-3-7-sonnet".to_string(),
                    tag: "[sonnet]".to_string(),
                    recommended: true,
                    note: "Anthropic Claude model".to_string(),
                },
                ModelOption {
                    id: "gpt-4o".to_string(),
                    tag: "[gpt-4o]".to_string(),
                    recommended: false,
                    note: "OpenAI flagship model".to_string(),
                },
            ],
            HarnessType::Plandex => vec![
                ModelOption {
                    id: "claude-3-7-sonnet".to_string(),
                    tag: "[sonnet]".to_string(),
                    recommended: true,
                    note: "Claude 3.7 reasoning".to_string(),
                },
                ModelOption {
                    id: "gpt-4o".to_string(),
                    tag: "[gpt-4o]".to_string(),
                    recommended: false,
                    note: "OpenAI flagship".to_string(),
                },
            ],
            HarnessType::Cline => vec![
                ModelOption {
                    id: "claude-3-7-sonnet".to_string(),
                    tag: "[sonnet]".to_string(),
                    recommended: true,
                    note: "Cline default model".to_string(),
                },
                ModelOption {
                    id: "gpt-4o".to_string(),
                    tag: "[openai]".to_string(),
                    recommended: false,
                    note: "OpenAI model".to_string(),
                },
            ],
            HarnessType::Smolagents => vec![
                ModelOption {
                    id: "Qwen/Qwen2.5-Coder-32B-Instruct".to_string(),
                    tag: "[qwen]".to_string(),
                    recommended: true,
                    note: "HuggingFace open model".to_string(),
                },
                ModelOption {
                    id: "meta-llama/Llama-3.3-70B-Instruct".to_string(),
                    tag: "[llama]".to_string(),
                    recommended: false,
                    note: "Meta Llama model".to_string(),
                },
            ],
            HarnessType::Mentat => vec![
                ModelOption {
                    id: "gpt-4o".to_string(),
                    tag: "[openai]".to_string(),
                    recommended: true,
                    note: "OpenAI default for Mentat".to_string(),
                },
                ModelOption {
                    id: "claude-3-7-sonnet".to_string(),
                    tag: "[sonnet]".to_string(),
                    recommended: false,
                    note: "Anthropic Claude".to_string(),
                },
            ],
            HarnessType::Custom => vec![],
        }
    }

    pub fn primary_api_key_name(&self) -> Option<&'static str> {
        match self {
            HarnessType::ClaudeCode => Some("ANTHROPIC_API_KEY"),
            HarnessType::GeminiCli => Some("GEMINI_API_KEY"),
            HarnessType::Antigravity => Some("GEMINI_API_KEY"),
            HarnessType::Codex => Some("OPENAI_API_KEY"),
            HarnessType::Aider => Some("ANTHROPIC_API_KEY"),
            HarnessType::Devin => Some("DEVIN_API_KEY"),
            HarnessType::December => Some("ANTHROPIC_API_KEY"),
            HarnessType::Pi => Some("INFLECTION_API_KEY"),
            HarnessType::CursorCli => Some("CURSOR_API_KEY"),
            HarnessType::SweAgent => Some("ANTHROPIC_API_KEY"),
            HarnessType::OpenHands => Some("ANTHROPIC_API_KEY"),
            HarnessType::Goose => Some("ANTHROPIC_API_KEY"),
            HarnessType::Plandex => Some("OPENAI_API_KEY"),
            HarnessType::Cline => Some("ANTHROPIC_API_KEY"),
            HarnessType::Smolagents => Some("HF_TOKEN"),
            HarnessType::Mentat => Some("OPENAI_API_KEY"),
            HarnessType::Custom => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelOption {
    pub id: String,
    pub tag: String,
    pub recommended: bool,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentProfile {
    pub harness: HarnessType,
    pub model: Option<String>,
    pub custom_cmd: Option<String>,
    pub custom_name: Option<String>,
    pub mount_dir: Option<String>,
}

impl AgentProfile {
    pub fn new(harness: HarnessType, model: Option<String>) -> Self {
        Self {
            harness,
            model,
            custom_cmd: None,
            custom_name: None,
            mount_dir: None,
        }
    }

    pub fn custom(cmd: String) -> Self {
        Self {
            harness: HarnessType::Custom,
            model: None,
            custom_cmd: Some(cmd),
            custom_name: None,
            mount_dir: None,
        }
    }

    pub fn custom_with_details(name: String, cmd: String, mount_dir: Option<String>) -> Self {
        Self {
            harness: HarnessType::Custom,
            model: None,
            custom_cmd: Some(cmd),
            custom_name: Some(name),
            mount_dir,
        }
    }

    pub fn from_name_or_cmd(name: Option<&str>, custom_cmd: Option<String>) -> Self {
        if let Some(cmd) = custom_cmd {
            if !cmd.trim().is_empty() {
                return AgentProfile::custom(cmd);
            }
        }

        let name_str = name.unwrap_or("claude-code").to_lowercase();
        match name_str.as_str() {
            "claude" | "claude-code" | "sonnet" => AgentProfile::new(
                HarnessType::ClaudeCode,
                Some("claude-3-7-sonnet-20250219".to_string()),
            ),
            "gemini" | "gemini-cli" => {
                AgentProfile::new(HarnessType::GeminiCli, Some("gemini-2.5-pro".to_string()))
            }
            "agy" | "antigravity" => {
                AgentProfile::new(HarnessType::Antigravity, Some("gemini-2.5-pro".to_string()))
            }
            "codex" | "openai-codex" => {
                AgentProfile::new(HarnessType::Codex, Some("gpt-4o".to_string()))
            }
            "aider" => AgentProfile::new(HarnessType::Aider, Some("claude-3-7-sonnet".to_string())),
            "devin" => AgentProfile::new(HarnessType::Devin, Some("devin-v1".to_string())),
            "december" => AgentProfile::new(HarnessType::December, Some("claude-3-7-sonnet".to_string())),
            "pi" => AgentProfile::new(HarnessType::Pi, Some("inflection-2.5".to_string())),
            "cursor" | "cursor-cli" => AgentProfile::new(HarnessType::CursorCli, Some("claude-3-7-sonnet".to_string())),
            "swe-agent" | "sweagent" => AgentProfile::new(
                HarnessType::SweAgent,
                Some("claude-3-7-sonnet-20250219".to_string()),
            ),
            "openhands" | "opendevin" => AgentProfile::new(
                HarnessType::OpenHands,
                Some("anthropic/claude-3-7-sonnet".to_string()),
            ),
            "goose" => AgentProfile::new(HarnessType::Goose, Some("claude-3-7-sonnet".to_string())),
            "plandex" => AgentProfile::new(HarnessType::Plandex, Some("claude-3-7-sonnet".to_string())),
            "cline" => AgentProfile::new(HarnessType::Cline, Some("claude-3-7-sonnet".to_string())),
            "smolagents" => AgentProfile::new(HarnessType::Smolagents, Some("Qwen/Qwen2.5-Coder-32B-Instruct".to_string())),
            "mentat" => AgentProfile::new(HarnessType::Mentat, Some("gpt-4o".to_string())),
            other => {
                if other.contains(' ') || other.contains('/') {
                    AgentProfile::custom(other.to_string())
                } else {
                    AgentProfile::new(HarnessType::ClaudeCode, Some(other.to_string()))
                }
            }
        }
    }

    pub fn name(&self) -> String {
        match &self.harness {
            HarnessType::ClaudeCode => {
                if let Some(m) = &self.model {
                    format!("Claude Code ({})", m)
                } else {
                    "Claude Code".to_string()
                }
            }
            HarnessType::GeminiCli => {
                if let Some(m) = &self.model {
                    format!("Gemini CLI ({})", m)
                } else {
                    "Gemini CLI".to_string()
                }
            }
            HarnessType::Antigravity => {
                if let Some(m) = &self.model {
                    format!("Antigravity ({})", m)
                } else {
                    "Antigravity".to_string()
                }
            }
            HarnessType::Codex => {
                if let Some(m) = &self.model {
                    format!("OpenAI Codex ({})", m)
                } else {
                    "OpenAI Codex".to_string()
                }
            }
            HarnessType::Aider => {
                if let Some(m) = &self.model {
                    format!("Aider ({})", m)
                } else {
                    "Aider".to_string()
                }
            }
            HarnessType::Devin => {
                if let Some(m) = &self.model {
                    format!("Devin ({})", m)
                } else {
                    "Devin".to_string()
                }
            }
            HarnessType::December => {
                if let Some(m) = &self.model {
                    format!("December ({})", m)
                } else {
                    "December".to_string()
                }
            }
            HarnessType::Pi => {
                if let Some(m) = &self.model {
                    format!("Pi ({})", m)
                } else {
                    "Pi".to_string()
                }
            }
            HarnessType::CursorCli => {
                if let Some(m) = &self.model {
                    format!("Cursor CLI ({})", m)
                } else {
                    "Cursor CLI".to_string()
                }
            }
            HarnessType::SweAgent => {
                if let Some(m) = &self.model {
                    format!("SWE-agent ({})", m)
                } else {
                    "SWE-agent".to_string()
                }
            }
            HarnessType::OpenHands => {
                if let Some(m) = &self.model {
                    format!("OpenHands ({})", m)
                } else {
                    "OpenHands".to_string()
                }
            }
            HarnessType::Goose => {
                if let Some(m) = &self.model {
                    format!("Goose ({})", m)
                } else {
                    "Goose".to_string()
                }
            }
            HarnessType::Plandex => {
                if let Some(m) = &self.model {
                    format!("Plandex ({})", m)
                } else {
                    "Plandex".to_string()
                }
            }
            HarnessType::Cline => {
                if let Some(m) = &self.model {
                    format!("Cline ({})", m)
                } else {
                    "Cline".to_string()
                }
            }
            HarnessType::Smolagents => {
                if let Some(m) = &self.model {
                    format!("Smolagents ({})", m)
                } else {
                    "Smolagents".to_string()
                }
            }
            HarnessType::Mentat => {
                if let Some(m) = &self.model {
                    format!("Mentat ({})", m)
                } else {
                    "Mentat".to_string()
                }
            }
            HarnessType::Custom => {
                if let Some(custom_name) = &self.custom_name {
                    custom_name.clone()
                } else if let Some(cmd) = &self.custom_cmd {
                    let trimmed = cmd.trim();
                    if trimmed.chars().count() > 30 {
                        let truncated: String = trimmed.chars().take(27).collect();
                        format!("Custom ({}...)", truncated)
                    } else {
                        format!("Custom ({})", trimmed)
                    }
                } else {
                    "Custom Agent".to_string()
                }
            }
        }
    }

    pub fn build_in_container_cmd(&self, _prompt: &str) -> String {
        match &self.harness {
            HarnessType::ClaudeCode => {
                if let Some(model) = &self.model {
                    format!(
                        "claude -p \"$SPACETIME_PROMPT\" --model \"{}\" --dangerously-skip-permissions",
                        model
                    )
                } else {
                    "claude -p \"$SPACETIME_PROMPT\" --dangerously-skip-permissions".to_string()
                }
            }
            HarnessType::GeminiCli => {
                if let Some(model) = &self.model {
                    format!(
                        "gemini --prompt \"$SPACETIME_PROMPT\" --model \"{}\" --yes --headless",
                        model
                    )
                } else {
                    "gemini --prompt \"$SPACETIME_PROMPT\" --yes --headless".to_string()
                }
            }
            HarnessType::Antigravity => {
                if let Some(model) = &self.model {
                    format!(
                        "agy --prompt \"$SPACETIME_PROMPT\" --model \"{}\" --yes",
                        model
                    )
                } else {
                    "agy --prompt \"$SPACETIME_PROMPT\" --yes".to_string()
                }
            }
            HarnessType::Codex => {
                if let Some(model) = &self.model {
                    format!(
                        "codex run --prompt \"$SPACETIME_PROMPT\" --model \"{}\"",
                        model
                    )
                } else {
                    "codex run --prompt \"$SPACETIME_PROMPT\"".to_string()
                }
            }
            HarnessType::Aider => {
                if let Some(model) = &self.model {
                    format!(
                        "aider --model \"{}\" --message \"$SPACETIME_PROMPT\" --yes --no-git --no-auto-commits",
                        model
                    )
                } else {
                    "aider --message \"$SPACETIME_PROMPT\" --yes --no-git --no-auto-commits".to_string()
                }
            }
            HarnessType::Devin => {
                "devin run --prompt \"$SPACETIME_PROMPT\"".to_string()
            }
            HarnessType::December => {
                if let Some(model) = &self.model {
                    format!("december run --prompt \"$SPACETIME_PROMPT\" --model \"{}\" --headless", model)
                } else {
                    "december run --prompt \"$SPACETIME_PROMPT\" --headless".to_string()
                }
            }
            HarnessType::Pi => {
                "pi query \"$SPACETIME_PROMPT\"".to_string()
            }
            HarnessType::CursorCli => {
                if let Some(model) = &self.model {
                    format!("cursor-agent --message \"$SPACETIME_PROMPT\" --model \"{}\"", model)
                } else {
                    "cursor-agent --message \"$SPACETIME_PROMPT\"".to_string()
                }
            }
            HarnessType::SweAgent => {
                if let Some(model) = &self.model {
                    format!(
                        "sweagent run --problem_statement \"$SPACETIME_PROMPT\" --model_name \"{}\"",
                        model
                    )
                } else {
                    "sweagent run --problem_statement \"$SPACETIME_PROMPT\"".to_string()
                }
            }
            HarnessType::OpenHands => {
                if let Some(model) = &self.model {
                    format!("openhands --model \"{}\" --task \"$SPACETIME_PROMPT\"", model)
                } else {
                    "openhands --task \"$SPACETIME_PROMPT\"".to_string()
                }
            }
            HarnessType::Goose => {
                if let Some(model) = &self.model {
                    format!("goose run --instruction \"$SPACETIME_PROMPT\" --model \"{}\"", model)
                } else {
                    "goose run --instruction \"$SPACETIME_PROMPT\"".to_string()
                }
            }
            HarnessType::Plandex => {
                if let Some(model) = &self.model {
                    format!("plandex prompt \"$SPACETIME_PROMPT\" --model \"{}\" --auto-apply", model)
                } else {
                    "plandex prompt \"$SPACETIME_PROMPT\" --auto-apply".to_string()
                }
            }
            HarnessType::Cline => {
                if let Some(model) = &self.model {
                    format!("cline run \"$SPACETIME_PROMPT\" --model \"{}\"", model)
                } else {
                    "cline run \"$SPACETIME_PROMPT\"".to_string()
                }
            }
            HarnessType::Smolagents => {
                if let Some(model) = &self.model {
                    format!("python3 -m smolagents.run --prompt \"$SPACETIME_PROMPT\" --model-id \"{}\"", model)
                } else {
                    "python3 -m smolagents.run --prompt \"$SPACETIME_PROMPT\"".to_string()
                }
            }
            HarnessType::Mentat => {
                if let Some(model) = &self.model {
                    format!("mentat --prompt \"$SPACETIME_PROMPT\" --model \"{}\" --auto", model)
                } else {
                    "mentat --prompt \"$SPACETIME_PROMPT\" --auto".to_string()
                }
            }
            HarnessType::Custom => {
                if let Some(template) = &self.custom_cmd {
                    if template.contains("\"{prompt}\"") {
                        template.replace("\"{prompt}\"", "\"$SPACETIME_PROMPT\"")
                    } else if template.contains("'{prompt}'") {
                        template.replace("'{prompt}'", "\"$SPACETIME_PROMPT\"")
                    } else if template.contains("{prompt}") {
                        template.replace("{prompt}", "\"$SPACETIME_PROMPT\"")
                    } else if template.contains("$SPACETIME_PROMPT") {
                        template.clone()
                    } else {
                        format!("{} \"$SPACETIME_PROMPT\"", template)
                    }
                } else {
                    "echo \"$SPACETIME_PROMPT\"".to_string()
                }
            }
        }
    }

    pub fn primary_api_key_name(&self) -> Option<&'static str> {
        if let Some(ref model) = self.model {
            let m_lower = model.to_lowercase();
            if m_lower.contains("gemini") {
                return Some("GEMINI_API_KEY");
            }
            if m_lower.contains("gpt") || m_lower.contains("o1") || m_lower.contains("o3") || m_lower.starts_with("openai/") {
                return Some("OPENAI_API_KEY");
            }
            if m_lower.contains("claude") || m_lower.contains("sonnet") || m_lower.contains("haiku") || m_lower.starts_with("anthropic/") {
                return Some("ANTHROPIC_API_KEY");
            }
            if m_lower.contains("deepseek") {
                return Some("DEEPSEEK_API_KEY");
            }
            if m_lower.contains("qwen") || m_lower.contains("llama") {
                return Some("HF_TOKEN");
            }
        }
        self.harness.primary_api_key_name()
    }

    pub fn check_env_status(&self) -> (String, bool) {
        if let Some(key) = self.primary_api_key_name() {
            if key == "GEMINI_API_KEY" {
                if std::env::var("GEMINI_API_KEY").is_ok() {
                    return ("GEMINI_API_KEY detected in .env".to_string(), true);
                }
                if std::env::var("GOOGLE_API_KEY").is_ok() {
                    return ("GOOGLE_API_KEY detected in .env".to_string(), true);
                }
                return ("GEMINI_API_KEY / GOOGLE_API_KEY not set".to_string(), false);
            }

            if let Ok(val) = std::env::var(key) {
                if !val.trim().is_empty() {
                    return (format!("{} detected in .env", key), true);
                }
            }
            (format!("{} not set in environment", key), false)
        } else {
            ("No standard API key required".to_string(), true)
        }
    }

    pub fn get_environment_variables(&self) -> Vec<String> {
        let mut envs = vec![
            "CI=true".to_string(),
            "TERM=xterm-256color".to_string(),
            "DEBIAN_FRONTEND=noninteractive".to_string(),
            "USER=agent".to_string(),
            "HOME=/home/agent".to_string(),
            "LOGNAME=agent".to_string(),
        ];

        let api_keys = [
            "ANTHROPIC_API_KEY",
            "GEMINI_API_KEY",
            "GOOGLE_API_KEY",
            "OPENAI_API_KEY",
            "DEEPSEEK_API_KEY",
            "GROQ_API_KEY",
            "OPENROUTER_API_KEY",
            "DATABRICKS_API_KEY",
            "DEVIN_API_KEY",
            "CURSOR_API_KEY",
            "INFLECTION_API_KEY",
            "HF_TOKEN",
        ];

        for key in api_keys {
            if let Ok(val) = std::env::var(key) {
                if !val.trim().is_empty() {
                    envs.push(format!("{}={}", key, val.trim()));
                }
            }
        }

        envs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_resolution() {
        let claude = AgentProfile::from_name_or_cmd(Some("claude"), None);
        assert_eq!(claude.harness, HarnessType::ClaudeCode);

        let gemini = AgentProfile::from_name_or_cmd(Some("gemini-cli"), None);
        assert_eq!(gemini.harness, HarnessType::GeminiCli);

        let agy = AgentProfile::from_name_or_cmd(Some("agy"), None);
        assert_eq!(agy.harness, HarnessType::Antigravity);

        let codex = AgentProfile::from_name_or_cmd(Some("codex"), None);
        assert_eq!(codex.harness, HarnessType::Codex);

        let devin = AgentProfile::from_name_or_cmd(Some("devin"), None);
        assert_eq!(devin.harness, HarnessType::Devin);

        let december = AgentProfile::from_name_or_cmd(Some("december"), None);
        assert_eq!(december.harness, HarnessType::December);

        let custom = AgentProfile::from_name_or_cmd(None, Some("python my_agent.py".to_string()));
        assert_eq!(custom.harness, HarnessType::Custom);
    }

    #[test]
    fn test_command_generation() {
        let profile = AgentProfile::new(
            HarnessType::ClaudeCode,
            Some("claude-3-7-sonnet-20250219".to_string()),
        );
        let cmd = profile.build_in_container_cmd("Fix nginx config");
        assert!(cmd.contains("claude -p \"$SPACETIME_PROMPT\""));
        assert!(cmd.contains("--model \"claude-3-7-sonnet-20250219\""));
        assert!(cmd.contains("--dangerously-skip-permissions"));

        let codex_profile = AgentProfile::new(
            HarnessType::Codex,
            Some("gpt-4o".to_string()),
        );
        let codex_cmd = codex_profile.build_in_container_cmd("Fix issue");
        assert!(codex_cmd.contains("codex run --prompt \"$SPACETIME_PROMPT\""));

        let custom_profile = AgentProfile::custom("python3 /workspace/agent.py {prompt}".to_string());
        let custom_cmd = custom_profile.build_in_container_cmd("Fix bug; rm -rf /");
        assert_eq!(custom_cmd, "python3 /workspace/agent.py \"$SPACETIME_PROMPT\"");
    }
}
