mod agent;
mod docker;
mod eval;
mod runner;
mod task;
mod types;
pub mod tui;

use std::path::{Path, PathBuf};
use anyhow::Result;
use clap::{Parser, Subcommand};
use comfy_table::{Cell, Color, Row, Table, presets::UTF8_FULL};

use crate::agent::AgentProfile;
use crate::docker::{DEFAULT_SANDBOX_IMAGE, ensure_sandbox_image};
use crate::eval::run_benchmark_suite;
use crate::runner::TaskRunner;
use crate::task::{find_task_by_id, load_all_tasks};
use crate::tui::run_spacetime_wizard;
use crate::tui::theme::{muted, orange, print_banner, trunk, white};

#[derive(Parser)]
#[command(
    name = "spacetime",
    author = "Chaitanya",
    version = "0.2.0",
    about = "An in-container benchmark arena for terminal AI agents (Claude Code, Gemini CLI, Aider, OpenHands, etc.)",
    long_about = "Spacetime evaluates AI agents by executing them directly inside hermetic Docker sandboxes on realistic Linux sysadmin and terminal tasks."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Task ID to run directly (e.g., 001-nginx-config or 001)
    #[arg(global = true)]
    task_id: Option<String>,

    /// Agent profile to benchmark (e.g. claude, claude-code, gemini, gemini-cli, aider, openhands)
    #[arg(short, long, global = true)]
    agent: Option<String>,

    /// Custom in-container agent CLI command template (e.g. "python3 /root/agent.py {prompt}")
    #[arg(long, global = true)]
    agent_cmd: Option<String>,

    /// Docker sandbox image tag (defaults to spacetime-sandbox:latest)
    #[arg(short, long, default_value = DEFAULT_SANDBOX_IMAGE, global = true)]
    image: String,

    /// Execution timeout override in seconds for the agent per task
    #[arg(long, global = true)]
    timeout: Option<u64>,

    /// Force rebuild of the Docker sandbox image
    #[arg(long, global = true)]
    force_rebuild: bool,

    /// Directory containing benchmark tasks
    #[arg(short = 't', long, default_value = "tasks", global = true)]
    tasks_dir: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the interactive TUI benchmark wizard
    Tui,

    /// Run a specific benchmark task
    Run {
        /// Task ID (e.g., 001-nginx-config, 001, port-conflict)
        task_id: String,
    },

    /// Run the full benchmark suite across all tasks
    EvalAll {
        /// Optional path to export JSON results
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Build or rebuild the Docker sandbox image
    BuildImage {
        /// Force rebuild even if image already exists
        #[arg(short, long)]
        force: bool,
    },

    /// List all available benchmark tasks
    List,

    /// Show detailed info and prompt for a specific task
    Info {
        /// Task ID
        task_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();
    let tasks_dir = &cli.tasks_dir;
    let agent_profile = AgentProfile::from_name_or_cmd(cli.agent.as_deref(), cli.agent_cmd);

    match cli.command {
        Some(Commands::Tui) => {
            run_spacetime_wizard(tasks_dir, cli.image, cli.timeout, cli.force_rebuild).await?;
        }
        Some(Commands::BuildImage { force }) => {
            ensure_sandbox_image(&cli.image, force || cli.force_rebuild).await?;
        }
        Some(Commands::List) => {
            list_tasks(tasks_dir)?;
        }
        Some(Commands::Info { task_id }) => {
            show_task_info(tasks_dir, &task_id)?;
        }
        Some(Commands::Run { task_id }) => {
            ensure_sandbox_image(&cli.image, cli.force_rebuild).await?;
            run_single_task(tasks_dir, &task_id, &agent_profile, &cli.image, cli.timeout).await?;
        }
        Some(Commands::EvalAll { output }) => {
            ensure_sandbox_image(&cli.image, cli.force_rebuild).await?;
            let tasks = load_all_tasks(tasks_dir)?;
            run_benchmark_suite(tasks, agent_profile, cli.image, cli.timeout, output).await?;
        }
        None => {
            if let Some(task_id) = cli.task_id {
                ensure_sandbox_image(&cli.image, cli.force_rebuild).await?;
                run_single_task(tasks_dir, &task_id, &agent_profile, &cli.image, cli.timeout).await?;
            } else {
                run_spacetime_wizard(tasks_dir, cli.image, cli.timeout, cli.force_rebuild).await?;
            }
        }
    }

    Ok(())
}

fn list_tasks(tasks_dir: &Path) -> Result<()> {
    let tasks = load_all_tasks(tasks_dir)?;
    println!(
        "\n{} {}",
        orange("✱"),
        white(&format!("found {} benchmark tasks in '{}':", tasks.len(), tasks_dir.display()))
    );
    println!("{}", trunk("│"));

    for (index, t) in tasks.iter().enumerate() {
        let is_last = index == tasks.len() - 1;
        let prefix = if is_last { "└─" } else { "├─" };
        let tag = crate::tui::tasks::get_task_category_tag(&t.id);
        println!(
            "{} {} {:<24} {:<6} {}",
            trunk("│"),
            trunk(prefix),
            white(&t.id),
            orange(tag),
            muted(&t.description)
        );
    }
    println!("{}", trunk("│"));
    Ok(())
}

fn show_task_info(tasks_dir: &Path, task_id: &str) -> Result<()> {
    let task = find_task_by_id(tasks_dir, task_id)?;
    println!("\n{}", trunk("────────────────────────────────────────────────────────"));
    println!(
        "{} {}",
        orange("✱"),
        white(&format!("task: {} ({})", task.name, task.id))
    );
    println!("{}\n", trunk("────────────────────────────────────────────────────────"));
    println!("  {:<14} {}", muted("description:"), white(&task.description));
    println!("  {:<14} {}", muted("max turns:"), white(&task.max_turns.to_string()));
    println!("  {:<14} {}s", muted("timeout:"), white(&task.timeout_secs.to_string()));
    println!("  {:<14} {}", muted("task dir:"), muted(&task.task_dir.display().to_string()));

    println!("\n{}", orange("• prompt given to agent:"));
    println!("  {}", white(&task.prompt));

    if task.setup_script.exists() {
        println!("\n{}", orange("• setup script (setup.sh):"));
        let setup_content = std::fs::read_to_string(&task.setup_script)?;
        for line in setup_content.lines() {
            println!("  {}", muted(line));
        }
    }

    if task.test_script.exists() {
        println!("\n{}", orange("• validation script (test.sh):"));
        let test_content = std::fs::read_to_string(&task.test_script)?;
        for line in test_content.lines() {
            println!("  {}", muted(line));
        }
    }

    Ok(())
}

async fn run_single_task(
    tasks_dir: &Path,
    task_id: &str,
    agent_profile: &AgentProfile,
    image: &str,
    timeout: Option<u64>,
) -> Result<()> {
    print_banner();
    let task = find_task_by_id(tasks_dir, task_id)?;
    TaskRunner::run_task(&task, agent_profile, image, timeout, false).await?;
    Ok(())
}
