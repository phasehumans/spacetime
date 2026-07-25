mod cli;
mod config;
mod embedded;
mod sandbox;
mod task;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use config::ConfigResolver;
use embedded::TaskLoader;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let app_config = ConfigResolver::resolve(
        cli.provider.clone(),
        cli.model.clone(),
        cli.api_key.clone(),
        None,
    )?;

    let tasks = if let Some(ref tasks_dir) = cli.tasks {
        TaskLoader::load_from_directory(tasks_dir)?
    } else {
        TaskLoader::load_embedded()?
    };

    match cli.command.unwrap_or(Commands::List) {
        Commands::List => {
            println!("⚡ Spacetime Benchmark Tasks ({})", tasks.len());
            println!("{:<12} {:<30} {:<15} {:<10}", "ID", "NAME", "IMAGE", "TURNS");
            println!("{}", "-".repeat(70));
            for t in &tasks {
                println!(
                    "{:<12} {:<30} {:<15} {:<10}",
                    t.id, t.name, t.base_image, t.max_turns
                );
            }
        }
        Commands::Config => {
            println!("⚙️  Spacetime Resolved Configuration:");
            println!("{}", serde_json::to_string_pretty(&app_config)?);
        }
        Commands::Eval { task, json } => {
            let task_id = task.as_deref().unwrap_or("all");
            if json {
                println!(
                    "{{\"status\":\"ready\",\"task\":\"{}\",\"provider\":\"{}\",\"model\":\"{}\"}}",
                    task_id, app_config.provider, app_config.model
                );
            } else {
                println!("🚀 Spacetime Agent Evaluation Harness");
                println!("Target Task: {}", task_id);
                println!("LLM Provider: {} ({})", app_config.provider, app_config.model);
            }
        }
    }

    Ok(())
}
