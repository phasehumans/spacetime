use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "spacetime")]
#[command(author = "Phase Humans")]
#[command(version = "0.1.0")]
#[command(about = "Minimalist framework for evaluating autonomous AI agents in isolated Docker sandboxes", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// LLM provider (openai, anthropic, gemini, ollama, openrouter, custom)
    #[arg(short, long, env = "SPACETIME_PROVIDER")]
    pub provider: Option<String>,

    /// LLM model name (e.g. gpt-4o, claude-3-5-sonnet-20241022, gemini-1.5-pro, llama3)
    #[arg(short, long, env = "SPACETIME_MODEL")]
    pub model: Option<String>,

    /// API key for the chosen LLM provider
    #[arg(short, long)]
    pub api_key: Option<String>,

    /// Directory containing custom task .sh files
    #[arg(long)]
    pub tasks: Option<PathBuf>,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    /// List all available benchmark tasks (embedded and custom)
    List,

    /// Evaluate an AI agent on a specific task or full suite
    Eval {
        /// Task ID or path to run (e.g. task-001)
        #[arg(short, long)]
        task: Option<String>,

        /// Run evaluation in non-interactive headless CLI mode with JSON output
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Display resolved Spacetime configuration
    Config,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_list() {
        let args = vec!["spacetime", "list"];
        let cli = Cli::parse_from(args);
        assert_eq!(cli.command, Some(Commands::List));
    }

    #[test]
    fn test_cli_parse_eval() {
        let args = vec!["spacetime", "eval", "--task", "task-001", "--json"];
        let cli = Cli::parse_from(args);
        assert_eq!(
            cli.command,
            Some(Commands::Eval {
                task: Some("task-001".to_string()),
                json: true,
            })
        );
    }
}
