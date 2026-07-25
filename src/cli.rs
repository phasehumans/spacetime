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

    #[arg(short, long, env = "SPACETIME_PROVIDER")]
    pub provider: Option<String>,

    #[arg(short, long, env = "SPACETIME_MODEL")]
    pub model: Option<String>,

    #[arg(short, long)]
    pub api_key: Option<String>,

    #[arg(long)]
    pub tasks: Option<PathBuf>,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    List,
    Eval {
        #[arg(short, long)]
        task: Option<String>,

        #[arg(long, default_value_t = false)]
        json: bool,

        #[arg(long, default_value_t = false)]
        full_screen: bool,
    },
    Config,
}
