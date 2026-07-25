mod cli;
mod cli_ui;
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
use cli_ui::{print_banner, select_task_interactively, TerminalObserver};
use config::ConfigResolver;
use colored::*;
use embedded::TaskLoader;
use engine::EvaluationEngine;
use provider::create_provider;
use sandbox::SandboxRuntime;
use tui::TuiDashboard;

fn handle_evaluation_error(err: anyhow::Error) {
    let msg = err.to_string();
    println!("\n{}", "Notice: Unable to complete evaluation session".bold().white());
    println!("  {}", msg.dimmed());

    if msg.contains("API key") || msg.contains("OPENAI_API_KEY") {
        println!("\n{}", "Quick Solutions:".white());
        println!("{}", "  • Set your OpenAI API key:      export OPENAI_API_KEY=\"your_key\"".dimmed());
        println!("{}", "  • Or run with local Ollama:     spacetime eval --provider ollama --model llama3".dimmed());
        println!("{}", "  • Or set key via CLI flag:      spacetime eval --api-key \"your_key\"\n".dimmed());
    }
}

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

    match cli.command {
        Some(Commands::List) => {
            print_banner();
            println!("{}", format!("Spacetime Benchmark Tasks ({})", tasks.len()).bold());
            println!("{:<12} {:<30} {:<15} {:<10}", "ID", "NAME", "IMAGE", "TURNS");
            println!("{}", "-".repeat(70));
            for t in &tasks {
                println!(
                    "{:<12} {:<30} {:<15} {:<10}",
                    t.id.bold(),
                    t.name,
                    t.base_image.dimmed(),
                    t.max_turns
                );
            }
        }
        Some(Commands::Config) => {
            print_banner();
            println!("{}", "Spacetime Resolved Configuration:".bold());
            println!("{}", serde_json::to_string_pretty(&app_config)?);
        }
        Some(Commands::Eval {
            task,
            json,
            full_screen,
        }) => {
            let target_task = if let Some(id) = task {
                tasks
                    .iter()
                    .find(|t| t.id == id)
                    .ok_or_else(|| anyhow!("Task '{}' not found", id))?
            } else {
                select_task_interactively(&tasks)?
            };

            let provider = create_provider(&app_config)?;
            let runtime = SandboxRuntime::new()?;

            if json {
                match EvaluationEngine::run_evaluation(
                    target_task,
                    provider.as_ref(),
                    &runtime,
                    &app_config,
                    None,
                )
                .await {
                    Ok(scorecard) => println!("{}", serde_json::to_string_pretty(&scorecard)?),
                    Err(e) => handle_evaluation_error(e),
                }
            } else if full_screen {
                let mut dashboard = TuiDashboard::new(
                    target_task.id.clone(),
                    app_config.provider.clone(),
                    app_config.model.clone(),
                    target_task.max_turns,
                )?;

                dashboard.start()?;
                let res = EvaluationEngine::run_evaluation(
                    target_task,
                    provider.as_ref(),
                    &runtime,
                    &app_config,
                    Some(&mut dashboard),
                )
                .await;
                dashboard.stop()?;

                match res {
                    Ok(scorecard) => {
                        println!("\nEvaluation Finished!");
                        println!("Result: {}", if scorecard.passed { "PASSED" } else { "FAILED" });
                        println!("Turns Used: {} / {}", scorecard.turns_used, scorecard.max_turns);
                        println!("Commands Executed: {}", scorecard.commands_executed);
                        println!("Duration: {}s", scorecard.duration_seconds);
                    }
                    Err(e) => handle_evaluation_error(e),
                }
            } else {
                print_banner();
                println!("[Provider] {}", app_config.provider.dimmed());
                println!("[Model]    {}", app_config.model.dimmed());
                println!("\n[Task]     {} - {}", target_task.id.bold(), target_task.name.bold());
                println!("• Objective:\n  {}", target_task.prompt.white());

                let mut observer = TerminalObserver::new();
                match EvaluationEngine::run_evaluation(
                    target_task,
                    provider.as_ref(),
                    &runtime,
                    &app_config,
                    Some(&mut observer),
                )
                .await {
                    Ok(scorecard) => {
                        println!("\n{}", "=".repeat(60).dimmed());
                        if scorecard.passed {
                            println!("{}", "  Evaluation PASSED".bold().white());
                        } else {
                            println!("{}", "  Evaluation FAILED".bold().dimmed());
                        }
                        println!("  Task: {}", scorecard.task_id.dimmed());
                        println!("  Turns Used: {} / {}", scorecard.turns_used, scorecard.max_turns.to_string().dimmed());
                        println!("  Commands Executed: {}", scorecard.commands_executed.to_string().dimmed());
                        println!("  Duration: {}s", scorecard.duration_seconds.to_string().dimmed());
                        println!("{}\n", "=".repeat(60).dimmed());
                    }
                    Err(e) => handle_evaluation_error(e),
                }
            }
        }
        None => {
            let target_task = select_task_interactively(&tasks)?;
            let provider = create_provider(&app_config)?;
            let runtime = SandboxRuntime::new()?;

            println!("[Provider] {}", app_config.provider.dimmed());
            println!("[Model]    {}", app_config.model.dimmed());
            println!("\n[Task]     {} - {}", target_task.id.bold(), target_task.name.bold());
            println!("• Objective:\n  {}", target_task.prompt.white());

            let mut observer = TerminalObserver::new();
            match EvaluationEngine::run_evaluation(
                target_task,
                provider.as_ref(),
                &runtime,
                &app_config,
                Some(&mut observer),
            )
            .await {
                Ok(scorecard) => {
                    println!("\n{}", "=".repeat(60).dimmed());
                    if scorecard.passed {
                        println!("{}", "  Evaluation PASSED".bold().white());
                    } else {
                        println!("{}", "  Evaluation FAILED".bold().dimmed());
                    }
                    println!("  Task: {}", scorecard.task_id.dimmed());
                    println!("  Turns Used: {} / {}", scorecard.turns_used, scorecard.max_turns.to_string().dimmed());
                    println!("  Commands Executed: {}", scorecard.commands_executed.to_string().dimmed());
                    println!("  Duration: {}s", scorecard.duration_seconds.to_string().dimmed());
                    println!("{}\n", "=".repeat(60).dimmed());
                }
                Err(e) => handle_evaluation_error(e),
            }
        }
    }

    Ok(())
}
