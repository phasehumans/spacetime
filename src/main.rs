mod cli;
mod config;
mod embedded;
mod engine;
mod provider;
mod sandbox;
mod task;
mod tui;

use anyhow::{anyhow, Result};
use clap::Parser;
use cli::{Cli, Commands};
use config::ConfigResolver;
use embedded::TaskLoader;
use engine::EvaluationEngine;
use provider::create_provider;
use sandbox::SandboxRuntime;
use tui::TuiDashboard;

#[tokio::main]
async fn main() -> Result<()> {
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
            let task_id = task.as_deref().unwrap_or("task-001");
            let target_task = tasks
                .iter()
                .find(|t| t.id == task_id)
                .ok_or_else(|| anyhow!("Task '{}' not found", task_id))?;

            let provider = create_provider(&app_config)?;
            let runtime = SandboxRuntime::new()?;

            if json {
                let scorecard = EvaluationEngine::run_evaluation(
                    target_task,
                    provider.as_ref(),
                    &runtime,
                    &app_config,
                    None,
                )
                .await?;
                println!("{}", serde_json::to_string_pretty(&scorecard)?);
            } else {
                let mut dashboard = TuiDashboard::new(
                    target_task.id.clone(),
                    app_config.provider.clone(),
                    app_config.model.clone(),
                    target_task.max_turns,
                )?;

                dashboard.start()?;
                let scorecard = EvaluationEngine::run_evaluation(
                    target_task,
                    provider.as_ref(),
                    &runtime,
                    &app_config,
                    Some(&mut dashboard),
                )
                .await?;
                dashboard.stop()?;

                println!("\n🏆 Evaluation Finished!");
                println!("Result: {}", if scorecard.passed { "PASSED ✅" } else { "FAILED ❌" });
                println!("Turns Used: {} / {}", scorecard.turns_used, scorecard.max_turns);
                println!("Commands Executed: {}", scorecard.commands_executed);
                println!("Duration: {}s", scorecard.duration_seconds);
            }
        }
    }

    Ok(())
}
