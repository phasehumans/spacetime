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

use crate::agent::AgentProfile;
use crate::docker::{DEFAULT_SANDBOX_IMAGE, ensure_sandbox_image};
use crate::eval::run_benchmark_suite;
use crate::runner::TaskRunner;
use crate::task::{find_task_by_id, load_all_tasks};
use crate::tui::run_spacetime_wizard;
use crate::tui::theme::{coral_red, muted, orange, print_banner, trunk, white};

#[derive(Parser)]
#[command(
    name = "spacetime",
    author = "Chaitanya",
    version = "0.0.2",
    about = "An in-container benchmark arena for terminal AI agents (Claude Code, Gemini CLI, Aider, OpenHands, etc.)",
    long_about = "Spacetime evaluates AI agents by executing them directly inside hermetic Docker sandboxes on realistic Linux sysadmin and terminal tasks."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(global = true)]
    task_id: Option<String>,

    #[arg(short, long, global = true)]
    agent: Option<String>,

    #[arg(long, global = true)]
    agent_cmd: Option<String>,

    #[arg(short, long, default_value = DEFAULT_SANDBOX_IMAGE, global = true)]
    image: String,

    #[arg(long, global = true)]
    timeout: Option<u64>,

    #[arg(long, global = true)]
    force_rebuild: bool,

    #[arg(short = 't', long, default_value = "tasks", global = true)]
    tasks_dir: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    Tui,

    Run {
        task_id: String,
    },

    EvalAll {
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    BuildImage {
        #[arg(short, long)]
        force: bool,
    },

    List,

    Clean,

    Info {
        task_id: String,
    },

    CreateTask {
        name: String,
        #[arg(short, long)]
        description: Option<String>,
        #[arg(short = 't', long, default_value = "tasks")]
        tasks_dir: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tokio::spawn(async {
        if tokio::signal::ctrl_c().await.is_ok() {
            eprintln!("\nReceived Ctrl+C / SIGINT. Cleaning up active sandbox containers...");
            crate::docker::cleanup_all_active_containers().await;
            crate::tui::theme::show_cursor();
            std::process::exit(130);
        }
    });

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
        Some(Commands::Clean) => {
            clean_sandbox_containers().await?;
        }
        Some(Commands::Info { task_id }) => {
            show_task_info(tasks_dir, &task_id)?;
        }
        Some(Commands::CreateTask { name, description, tasks_dir: custom_tasks_dir }) => {
            create_benchmark_task(&name, description.as_deref(), &custom_tasks_dir)?;
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

async fn clean_sandbox_containers() -> Result<()> {
    use bollard::Docker;
    use bollard::container::{ListContainersOptions, RemoveContainerOptions};
    use std::collections::HashMap;

    let docker = Docker::connect_with_local_defaults()?;
    let mut filters = HashMap::new();
    filters.insert("name".to_string(), vec!["spacetime-".to_string()]);

    let containers = docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        }))
        .await?;

    println!(
        "\n{}  {}",
        orange("✱"),
        white("cleaning spacetime sandbox containers...")
    );
    println!("{}", trunk("│"));

    if containers.is_empty() {
        println!(
            "{}  {}",
            trunk("│"),
            muted("no active or dangling spacetime containers found")
        );
    } else {
        for c in containers {
            if let Some(id) = c.id {
                let _ = docker
                    .remove_container(
                        &id,
                        Some(RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                    )
                    .await;
                println!(
                    "{}  {} {}",
                    trunk("│"),
                    coral_red("removed:"),
                    muted(&id.chars().take(12).collect::<String>())
                );
            }
        }
    }
    println!("{}", trunk("│"));
    println!("{}  {}", orange("✱"), white("clean state verified"));
    println!("{}", trunk("│"));
    Ok(())
}

fn list_tasks(tasks_dir: &Path) -> Result<()> {
    let tasks = load_all_tasks(tasks_dir)?;
    println!(
        "\n{}  {}",
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
        "{}  {}",
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

fn create_benchmark_task(name: &str, description: Option<&str>, tasks_dir: &Path) -> Result<()> {
    use std::fs;
    use crate::tui::theme::mint_green;

    let clean_name = name
        .to_lowercase()
        .replace([' ', '_'], "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    if clean_name.is_empty() {
        return Err(anyhow::anyhow!("Invalid task name"));
    }

    if !tasks_dir.exists() {
        fs::create_dir_all(tasks_dir)?;
    }

    let mut highest_idx = 0;
    if let Ok(entries) = fs::read_dir(tasks_dir) {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if let Some((prefix, _)) = filename.split_once('-') {
                if let Ok(idx) = prefix.parse::<usize>() {
                    if idx > highest_idx {
                        highest_idx = idx;
                    }
                }
            }
        }
    }

    let next_idx = highest_idx + 1;
    let folder_name = format!("{:03}-{}", next_idx, clean_name);
    let target_dir = tasks_dir.join(&folder_name);

    if target_dir.exists() {
        return Err(anyhow::anyhow!("Task directory already exists: {}", target_dir.display()));
    }

    fs::create_dir_all(&target_dir)?;

    let task_desc = description.unwrap_or("Task created with spacetime create-task");

    let prompt_content = format!("{}\n", task_desc);
    fs::write(target_dir.join("prompt.txt"), prompt_content)?;

    let setup_content = format!(
        "#!/usr/bin/env bash\nset -e\n\n# Setup broken environment state for {}\necho \"Setting up environment for {}...\"\n",
        folder_name, folder_name
    );
    fs::write(target_dir.join("setup.sh"), setup_content)?;

    let test_content = "#!/usr/bin/env bash\nset -e\n\n# Ground-truth verification assertions (exit 0 = pass, exit 1 = fail)\necho \"Running ground-truth tests...\"\nexit 0\n";
    fs::write(target_dir.join("test.sh"), test_content)?;

    let meta_content = format!(
        "TASK_ID=\"{}\"\nTASK_NAME=\"{}\"\nBASE_IMAGE=\"ubuntu:22.04\"\nMAX_TURNS=15\nTIMEOUT_SECS=60\nDESCRIPTION=\"{}\"\n",
        folder_name, name, task_desc
    );
    fs::write(target_dir.join("meta.sh"), meta_content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(target_dir.join("setup.sh")) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(target_dir.join("setup.sh"), perms);
        }
        if let Ok(metadata) = fs::metadata(target_dir.join("test.sh")) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(target_dir.join("test.sh"), perms);
        }
    }

    println!("\n{}", orange("✱  task scaffold created successfully!"));
    println!("  {:<12} {}", white("folder:"), mint_green(&target_dir.display().to_string()));
    println!("  {:<12} {}", white("task id:"), white(&folder_name));
    println!("  {:<12} {}", white("prompt:"), muted(&target_dir.join("prompt.txt").display().to_string()));
    println!("  {:<12} {}", white("setup:"), muted(&target_dir.join("setup.sh").display().to_string()));
    println!("  {:<12} {}", white("test:"), muted(&target_dir.join("test.sh").display().to_string()));
    println!("\n{}", trunk("│  test your new task with:"));
    println!("{}  {}", trunk("│"), white(&format!("spacetime run {}", folder_name)));
    println!();

    Ok(())
}
