use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use anyhow::{Context, Result};
use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};

use crate::agent::AgentProfile;
use crate::eval::{compute_intelligence_profile, generate_harness_insights, print_evaluation_summary};
use crate::runner::{TaskRunner, TaskStage};
use crate::tui::theme::{
    coral_red, mint_green, muted, orange, trunk, white,
};
use crate::types::{BenchmarkSuiteResult, BenchmarkTask, TaskResult};

pub async fn execute_benchmark_suite_tui(
    tasks: Vec<BenchmarkTask>,
    agent_profile: AgentProfile,
    sandbox_image: String,
    timeout_override: Option<u64>,
    output_path: Option<PathBuf>,
) -> Result<BenchmarkSuiteResult> {
    let start_time = Instant::now();
    let total_tasks = tasks.len();
    let timestamp = Utc::now().to_rfc3339();
    let agent_name = agent_profile.name();

    println!(
        "{}  {}",
        orange("✱"),
        white(&format!("running benchmark suite ({} tasks)", total_tasks))
    );
    println!("{}", trunk("│"));

    let mut results: Vec<TaskResult> = Vec::new();

    for (index, task) in tasks.iter().enumerate() {
        let is_last = index == total_tasks - 1;
        let prefix = if is_last { "└─" } else { "├─" };

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
                .template("\x1b[38;2;63;63;70m│  \x1b[0m\x1b[38;2;251;146;60m{spinner}\x1b[0m \x1b[38;2;228;228;231m{msg}\x1b[0m")
                .unwrap_or_else(|_| ProgressStyle::default_spinner()),
        );
        spinner.enable_steady_tick(Duration::from_millis(80));

        let task_id = task.id.clone();
        let sp = spinner.clone();
        let task_start = Instant::now();

        let update_msg = move |stage: TaskStage| {
            let elapsed = task_start.elapsed().as_secs_f64();
            sp.set_message(format!(
                "{} {} {}",
                white(&task_id),
                muted(&format!("({})", stage.description())),
                muted(&format!("[{:.1}s]", elapsed))
            ));
        };

        update_msg(TaskStage::InitializingSandbox);

        let result = TaskRunner::run_task_with_progress(
            task,
            &agent_profile,
            &sandbox_image,
            timeout_override,
            true,
            update_msg,
        )
        .await?;

        spinner.finish_and_clear();

        if result.passed {
            let status_badge = mint_green("pass");
            println!(
                "{} {} {:<24} {}",
                trunk(&format!("│  {}", prefix)),
                status_badge,
                white(&task.id),
                muted(&format!("({:.1}s)", result.duration_secs))
            );
        } else {
            let status_badge = coral_red("fail");
            let err_snippet = if let Some(err) = &result.error_message {
                let clean = err.lines().next().unwrap_or("failed");
                if clean.len() > 36 {
                    format!("{}...", &clean[..33])
                } else {
                    clean.to_string()
                }
            } else {
                "test assertion failed".to_string()
            };

            println!(
                "{} {} {:<24} {} - {}",
                trunk(&format!("│  {}", prefix)),
                status_badge,
                white(&task.id),
                muted(&format!("({:.1}s)", result.duration_secs)),
                coral_red(&err_snippet)
            );
        }

        results.push(result);
    }

    let total_duration = start_time.elapsed().as_secs_f64();
    let passed_tasks = results.iter().filter(|r| r.passed).count();
    let failed_tasks = total_tasks - passed_tasks;
    let pass_rate = if total_tasks > 0 {
        (passed_tasks as f64 / total_tasks as f64) * 100.0
    } else {
        0.0
    };

    let intelligence_profile = compute_intelligence_profile(&tasks, &results);
    let harness_insights = generate_harness_insights(&tasks, &results, &intelligence_profile);

    let suite_result = BenchmarkSuiteResult {
        timestamp,
        agent: agent_name,
        sandbox: sandbox_image,
        total_tasks,
        passed_tasks,
        failed_tasks,
        pass_rate,
        total_duration_secs: total_duration,
        results,
        intelligence_profile: Some(intelligence_profile),
        harness_insights,
    };

    let export_file = output_path.unwrap_or_else(|| {
        let results_dir = Path::new("results");
        if !results_dir.exists() {
            let _ = fs::create_dir_all(results_dir);
        }
        results_dir.join(format!(
            "spacetime_results_{}.json",
            Utc::now().format("%Y%m%d_%H%M%S")
        ))
    });

    if let Some(parent) = export_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let json_bytes = serde_json::to_vec_pretty(&suite_result)
        .context("Failed to serialize benchmark results to JSON")?;
    fs::write(&export_file, json_bytes)
        .with_context(|| format!("Failed to write results file to {}", export_file.display()))?;

    print_evaluation_summary(&suite_result, &tasks, &export_file);

    Ok(suite_result)
}
