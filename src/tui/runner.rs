use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use anyhow::{Context, Result};
use chrono::Utc;
use indicatif::{ProgressBar, ProgressStyle};

use crate::agent::AgentProfile;
use crate::runner::{TaskRunner, TaskStage};
use crate::tui::tasks::get_task_category_tag;
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
        "{} {}",
        orange("◐"),
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
            let status_badge = format!("{} {}", mint_green("●"), mint_green("pass"));
            println!(
                "{} {} {:<24} {}",
                trunk(&format!("│  {}", prefix)),
                status_badge,
                white(&task.id),
                muted(&format!("({:.1}s)", result.duration_secs))
            );
        } else {
            let status_badge = format!("{} {}", coral_red("●"), coral_red("fail"));
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

    println!("{}", trunk("│"));
    let pass_str = format!("{} {} passed", mint_green("●"), passed_tasks);
    let fail_str = if failed_tasks > 0 {
        format!("{} {} failed", coral_red("●"), failed_tasks)
    } else {
        format!("0 failed")
    };

    println!(
        "{} {} {} {} {} {}",
        orange("✱"),
        white("evaluation summary:"),
        mint_green(&pass_str),
        muted(","),
        coral_red(&fail_str),
        muted(&format!("({:.1}% pass rate)", pass_rate))
    );
    println!(
        "{} {}",
        trunk("│"),
        muted(&format!(
            "total duration: {:.1}s | avg duration: {:.1}s/task",
            total_duration,
            if total_tasks > 0 {
                total_duration / total_tasks as f64
            } else {
                0.0
            }
        ))
    );

    // Category breakdown
    let mut category_map: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
    for (t, r) in tasks.iter().zip(results.iter()) {
        let tag = get_task_category_tag(&t.id);
        let entry = category_map.entry(tag).or_insert((0, 0));
        entry.0 += if r.passed { 1 } else { 0 };
        entry.1 += 1;
    }

    println!("{}", trunk("│"));
    println!("{} {}", orange("✱"), white("category breakdown:"));
    for (tag, (passed, count)) in category_map {
        let cat_rate = (passed as f64 / count as f64) * 100.0;
        let cat_name = match tag {
            "[net]" => "network & services",
            "[git]" => "git & versioning",
            "[sec]" => "security & perms",
            "[data]" => "data & formatting",
            "[fs]" => "filesystem operations",
            "[dev]" => "dev & environments",
            "[logs]" => "log analysis",
            _ => "general terminal tasks",
        };
        let mut bar = String::new();
        for i in 0..count {
            if i < passed {
                bar.push_str(&mint_green("●"));
            } else {
                bar.push_str(&coral_red("○"));
            }
        }
        println!(
            "{} {:<6} {:<22} {:>2}/{} ({:>5.1}%)  {}",
            trunk("│"),
            orange(tag),
            white(cat_name),
            passed,
            count,
            cat_rate,
            bar
        );
    }

    // Failure Diagnostics
    let failed_results: Vec<(&BenchmarkTask, &TaskResult)> = tasks
        .iter()
        .zip(results.iter())
        .filter(|(_, r)| !r.passed)
        .collect();

    if !failed_results.is_empty() {
        println!("{}", trunk("│"));
        println!("{} {}", orange("✱"), white("failure diagnostics:"));
        let num_failed = failed_results.len();

        for (idx, (t, r)) in failed_results.iter().enumerate() {
            let is_last_fail = idx == num_failed - 1;
            let branch = if is_last_fail { "└─" } else { "├─" };
            let sub_pipe = if is_last_fail { "  " } else { "│ " };

            let (reason, detail) = if let Some(ref err) = r.error_message {
                if err.to_lowercase().contains("timeout") {
                    (
                        "⏱ Timeout",
                        format!("exceeded timeout limit ({:.1}s)", r.duration_secs),
                    )
                } else if err.to_lowercase().contains("setup.sh") {
                    ("⚙ Setup Script Failed", err.clone())
                } else if err.to_lowercase().contains("api_key")
                    || err.to_lowercase().contains("401")
                {
                    ("⚠ API Key / Auth Error", err.clone())
                } else {
                    ("✗ Test Assertion Failed", err.clone())
                }
            } else {
                (
                    "✗ Test Assertion Failed",
                    "test.sh returned non-zero exit code".to_string(),
                )
            };

            let clean_agent = agent_name.replace([' ', '(', ')', '/', '\\', ':'], "_");
            let log_path = format!("results/logs/{}_{}_FAIL.log", t.id, clean_agent);

            println!("{} {} {}", trunk("│"), trunk(branch), white(&t.id));
            println!(
                "{} {} reason: {}",
                trunk("│"),
                trunk(sub_pipe),
                coral_red(reason)
            );
            println!(
                "{} {} detail: {}",
                trunk("│"),
                trunk(sub_pipe),
                muted(&detail)
            );
            println!(
                "{} {} log:    {}",
                trunk("│"),
                trunk(sub_pipe),
                muted(&log_path)
            );
            if !is_last_fail {
                println!("{} {}", trunk("│"), trunk(sub_pipe));
            }
        }
    }

    let suite_result = BenchmarkSuiteResult {
        timestamp,
        agent: agent_name,
        total_tasks,
        passed_tasks,
        failed_tasks,
        pass_rate,
        total_duration_secs: total_duration,
        results,
    };

    // Save JSON report
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

    println!("{}", trunk("│"));
    println!(
        "{} {}",
        trunk("│  report saved to:"),
        white(&export_file.display().to_string())
    );
    println!("{}", trunk("│"));

    Ok(suite_result)
}
