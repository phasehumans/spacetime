use std::fs;
use std::path::Path;
use std::time::Instant;
use anyhow::{Context, Result, anyhow};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use crate::agent::AgentProfile;
use crate::docker::EnvironmentManager;
use crate::types::{BenchmarkTask, TaskResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStage {
    InitializingSandbox,
    ExecutingSetup,
    AgentRunning,
    EvaluatingTest,
}

impl TaskStage {
    pub fn description(&self) -> &'static str {
        match self {
            TaskStage::InitializingSandbox => "initializing sandbox...",
            TaskStage::ExecutingSetup => "executing setup.sh...",
            TaskStage::AgentRunning => "agent coding & solving...",
            TaskStage::EvaluatingTest => "evaluating test.sh...",
        }
    }
}

pub struct TaskRunner;

impl TaskRunner {
    pub async fn run_task(
        task: &BenchmarkTask,
        agent_profile: &AgentProfile,
        sandbox_image: &str,
        timeout_override: Option<u64>,
        silent: bool,
    ) -> Result<TaskResult> {
        Self::run_task_with_progress(
            task,
            agent_profile,
            sandbox_image,
            timeout_override,
            silent,
            |_| {},
        )
        .await
    }

    pub async fn run_task_with_progress<F>(
        task: &BenchmarkTask,
        agent_profile: &AgentProfile,
        sandbox_image: &str,
        timeout_override: Option<u64>,
        silent: bool,
        mut on_stage: F,
    ) -> Result<TaskResult>
    where
        F: FnMut(TaskStage),
    {
        let start_time = Instant::now();
        let agent_name = agent_profile.name();

        if !silent {
            println!("{}", format!("\n[Task]     {}", task.name).bold().white());
            println!("{}", format!("[Agent]    {}", agent_name).cyan());
            println!("{}", "\n• Target Objective:".dimmed());
            println!("  {}", task.prompt.white());
        }

        on_stage(TaskStage::InitializingSandbox);
        let spinner = if !silent {
            Some(create_spinner("Initializing Docker sandbox..."))
        } else {
            None
        };

        let mut env = EnvironmentManager::with_mount(
            sandbox_image.to_string(),
            agent_profile.mount_dir.clone(),
        )?;

        let mut is_initialized = false;
        let mut error_message = None;
        let mut agent_output = String::new();
        let mut agent_exit_code = None;
        let mut passed = false;

        let eval_res = async {
            env.initialize()
                .await
                .with_context(|| "Sandbox initialization failed")?;
            is_initialized = true;

            on_stage(TaskStage::ExecutingSetup);
            if let Some(ref sp) = spinner {
                sp.set_message("Executing setup.sh inside sandbox...");
            }

            let setup_res = env.execute_host_script(&task.setup_script, 180).await?;
            if setup_res.exit_code != 0 {
                return Err(anyhow!(
                    "setup.sh failed with exit code {}: {}",
                    setup_res.exit_code,
                    setup_res.stderr
                ));
            }

            if let Some(sp) = spinner {
                sp.finish_and_clear();
            }

            if !silent {
                println!(
                    "{}",
                    format!("Environment initialized ({})", task.id).green()
                );
            }

            on_stage(TaskStage::AgentRunning);
            let agent_cmd = agent_profile.build_in_container_cmd(&task.prompt);
            let mut env_vars = agent_profile.get_environment_variables();
            env_vars.push(format!("SPACETIME_PROMPT={}", task.prompt));
            env_vars.push(format!("PROMPT={}", task.prompt));
            let effective_timeout = timeout_override.unwrap_or(300);

            if !silent {
                println!("\n{}", "• Spawning In-Container Agent:".bold().white());
                println!("  {}\n", agent_cmd.yellow());
                println!("{}", "--------------------------------------------------------".dimmed());
            }

            let agent_res = env
                .execute_agent_stream(&agent_cmd, &env_vars, effective_timeout, silent)
                .await?;

            if !silent {
                println!("\n{}", "--------------------------------------------------------".dimmed());
            }

            agent_output = crate::docker::scrub_secrets(&agent_res.stdout);
            agent_exit_code = Some(agent_res.exit_code);

            on_stage(TaskStage::EvaluatingTest);
            let test_spinner = if !silent {
                Some(create_spinner("Evaluating final container state with test.sh..."))
            } else {
                None
            };

            let test_res = env.execute_host_script(&task.test_script, 60).await?;
            if let Some(sp) = test_spinner {
                sp.finish_and_clear();
            }

            passed = test_res.exit_code == 0;
            Ok::<(), anyhow::Error>(())
        }
        .await;

        if let Err(ref e) = eval_res {
            error_message = Some(crate::docker::scrub_secrets(&e.to_string()));
            if !silent {
                eprintln!("{}", format!("\n[Error] {}", e).bright_red());
            }
        }

        if is_initialized {
            let _ = env.destroy().await;
        }

        let total_duration = start_time.elapsed().as_secs_f64();

        save_task_log(&task.id, &agent_profile.name(), &agent_output, passed)?;

        if !silent {
            println!("\n{}", "• SPACETIME EVALUATION •".bold());
            println!("  {:<12} {}", "Task:".dimmed(), task.name.white());
            println!("  {:<12} {}", "Agent:".dimmed(), agent_name.white());
            println!("  {:<12} {:.2}s", "Duration:".dimmed(), total_duration);
            if passed {
                println!("  {:<12} {}", "Result:".dimmed(), "PASSED".green().bold());
            } else {
                println!("  {:<12} {}", "Result:".dimmed(), "FAILED".red().bold());
            }
            println!("{}\n", "========================================================".dimmed());
        }

        Ok(TaskResult {
            task_id: task.id.clone(),
            task_name: task.name.clone(),
            passed,
            duration_secs: total_duration,
            agent_name,
            exit_code: agent_exit_code,
            error_message,
            agent_output,
        })
    }
}

fn save_task_log(task_id: &str, agent_name: &str, output: &str, passed: bool) -> Result<()> {
    let clean_agent = agent_name.replace([' ', '(', ')', '/', '\\', ':'], "_");
    let log_dir = Path::new("results").join("logs");
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)
            .with_context(|| format!("Failed to create log directory: {}", log_dir.display()))?;
    }

    let status = if passed { "PASS" } else { "FAIL" };
    let log_file = log_dir.join(format!("{}_{}_{}.log", task_id, clean_agent, status));
    let scrubbed_output = crate::docker::scrub_secrets(output);
    fs::write(&log_file, scrubbed_output)
        .with_context(|| format!("Failed to write log file: {}", log_file.display()))?;
    Ok(())
}

fn create_spinner(msg: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("\x1b[38;2;251;146;60m{spinner}\x1b[0m \x1b[38;2;228;228;231m{msg}\x1b[0m")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    spinner.set_message(msg.to_string());
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));
    spinner
}
