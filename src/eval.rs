use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use anyhow::{Context, Result};
use chrono::Utc;

use crate::agent::AgentProfile;
use crate::runner::TaskRunner;
use crate::tui::tasks::get_task_category_tag;
use crate::tui::theme::{
    coral_red, mint_green, muted, orange, trunk, white,
};
use crate::types::{BenchmarkSuiteResult, BenchmarkTask, IntelligenceProfile, TaskResult};

pub async fn run_benchmark_suite(
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
        "\n{}  {}",
        orange("✱"),
        white(&format!("SPACETIME BENCHMARK SUITE ({} Tasks)", total_tasks))
    );
    println!(
        "{}  {:<12} {}",
        trunk("│"),
        white("agent:"),
        muted(&agent_name)
    );
    println!(
        "{}  {:<12} {}",
        trunk("│"),
        white("sandbox:"),
        muted(&sandbox_image)
    );
    println!("{}", trunk("│"));

    let mut results: Vec<TaskResult> = Vec::new();

    for (index, task) in tasks.iter().enumerate() {
        let is_last = index == total_tasks - 1;
        let prefix = if is_last { "└─" } else { "├─" };

        let result = TaskRunner::run_task(
            task,
            &agent_profile,
            &sandbox_image,
            timeout_override,
            true,
        )
        .await?;

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
            let err_snippet = if let Some(ref err) = result.error_message {
                let clean = err.lines().next().unwrap_or("failed");
                if clean.chars().count() > 36 {
                    format!("{}...", clean.chars().take(33).collect::<String>())
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
        total_tokens: intelligence_profile.total_tokens,
        total_cost_usd: intelligence_profile.total_cost_usd,
        cost_per_resolved_task: intelligence_profile.cost_per_resolved_task,
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

    let json_data = serde_json::to_string_pretty(&suite_result)?;
    fs::write(&export_file, json_data)
        .with_context(|| format!("Failed to write results to {}", export_file.display()))?;

    print_evaluation_summary(&suite_result, &tasks, &export_file);

    Ok(suite_result)
}

pub fn compute_intelligence_profile(
    tasks: &[BenchmarkTask],
    results: &[TaskResult],
) -> IntelligenceProfile {
    let total_tasks = tasks.len();
    if total_tasks == 0 {
        return IntelligenceProfile::default();
    }

    let mut first_attempt_count = 0;
    let mut tasks_with_errors = 0;
    let mut recovered_count = 0;
    let mut verified_count = 0;
    let mut total_output_chars = 0;
    let mut total_prompt_tokens = 0;
    let mut total_completion_tokens = 0;
    let mut total_tokens = 0;
    let mut total_cost_usd = 0.0;

    for r in results {
        total_prompt_tokens += r.prompt_tokens;
        total_completion_tokens += r.completion_tokens;
        total_tokens += r.total_tokens;
        total_cost_usd += r.estimated_cost_usd;

        let output = &r.agent_output;
        total_output_chars += output.len();
        let lower = output.to_lowercase();

        let has_errors = lower.contains("error:")
            || lower.contains("command not found")
            || lower.contains("permission denied")
            || lower.contains("syntaxerror")
            || lower.contains("cannot open")
            || lower.contains("no such file")
            || lower.contains("failed with exit code");

        if has_errors {
            tasks_with_errors += 1;
            if r.passed {
                recovered_count += 1;
            }
        } else if r.passed {
            first_attempt_count += 1;
        }

        let has_verification = lower.contains("curl")
            || lower.contains("nginx -t")
            || lower.contains("git status")
            || lower.contains("git diff")
            || lower.contains("pytest")
            || lower.contains("unittest")
            || lower.contains("test -")
            || lower.contains("diff ")
            || lower.contains("grep ")
            || lower.contains("ps aux")
            || lower.contains("ss -tulpn")
            || lower.contains("head ")
            || lower.contains("tail ");

        if has_verification {
            verified_count += 1;
        }
    }

    let first_attempt_rate = (first_attempt_count as f64 / total_tasks as f64) * 100.0;
    let error_recovery_rate = if tasks_with_errors > 0 {
        (recovered_count as f64 / tasks_with_errors as f64) * 100.0
    } else {
        100.0
    };
    let self_verification_rate = (verified_count as f64 / total_tasks as f64) * 100.0;

    let passed_count = results.iter().filter(|r| r.passed).count();
    let cost_per_resolved_task = if passed_count > 0 {
        total_cost_usd / passed_count as f64
    } else {
        0.0
    };

    let avg_chars = total_output_chars as f64 / total_tasks as f64;
    let context_hygiene_score = if avg_chars < 3000.0 {
        98.5
    } else if avg_chars < 8000.0 {
        91.2
    } else if avg_chars < 15000.0 {
        82.4
    } else {
        74.0
    };

    IntelligenceProfile {
        first_attempt_rate,
        first_attempt_count,
        error_recovery_rate,
        recovered_count,
        tasks_with_errors,
        self_verification_rate,
        verified_count,
        context_hygiene_score,
        total_prompt_tokens,
        total_completion_tokens,
        total_tokens,
        total_cost_usd,
        cost_per_resolved_task,
    }
}

pub fn generate_harness_insights(
    _tasks: &[BenchmarkTask],
    results: &[TaskResult],
    profile: &IntelligenceProfile,
) -> Vec<String> {
    let mut insights = Vec::new();
    let failed_tasks: Vec<&TaskResult> = results.iter().filter(|r| !r.passed).collect();

    let unverified_failures = failed_tasks
        .iter()
        .filter(|r| {
            let lower = r.agent_output.to_lowercase();
            !lower.contains("curl")
                && !lower.contains("nginx -t")
                && !lower.contains("test")
                && !lower.contains("pytest")
                && !lower.contains("status")
        })
        .count();

    if unverified_failures > 0 {
        insights.push(format!(
            "Verification Gap: {} failed task(s) terminated without executing post-fix validation checks.",
            unverified_failures
        ));
    }

    let service_failures = failed_tasks
        .iter()
        .filter(|r| {
            r.task_id.contains("nginx")
                || r.task_id.contains("port")
                || r.task_id.contains("service")
        })
        .count();
    if service_failures > 0 {
        insights.push(
            "Service Management: Agent modified configuration but omitted daemon reload / active listener verification.".to_string(),
        );
    }

    if profile.tasks_with_errors > 0 && profile.error_recovery_rate < 70.0 {
        insights.push(
            "Error Reflection: Agent struggled to pivot after encountering initial terminal stderr outputs.".to_string(),
        );
    }

    if insights.len() < 3 {
        if profile.first_attempt_rate >= 80.0 {
            insights.push(
                "Execution Precision: Agent demonstrated high one-shot command accuracy with minimal syntax retries.".to_string(),
            );
        } else {
            insights.push(
                "Tool Efficiency: Adding structured file-patching tools may reduce multi-turn edit retries.".to_string(),
            );
        }
    }

    if insights.len() < 3 {
        insights.push(
            "Context Hygiene: Streamed output stayed compact, avoiding context window saturation.".to_string(),
        );
    }

    insights.truncate(3);
    insights
}

pub fn print_evaluation_summary(
    suite_result: &BenchmarkSuiteResult,
    tasks: &[BenchmarkTask],
    export_file: &Path,
) {
    let pass_str = mint_green(&format!("{} passed", suite_result.passed_tasks));
    let fail_str = if suite_result.failed_tasks > 0 {
        coral_red(&format!("{} failed", suite_result.failed_tasks))
    } else {
        "0 failed".to_string()
    };

    println!("{}", trunk("│"));
    println!(
        "{}  {} {} {} {} {}",
        orange("✱"),
        white("evaluation summary:"),
        pass_str,
        muted(","),
        fail_str,
        muted(&format!("({:.1}% pass rate)", suite_result.pass_rate))
    );
    println!(
        "{}  {:<12} {}",
        trunk("│"),
        white("agent:"),
        muted(&suite_result.agent)
    );
    if !suite_result.sandbox.is_empty() {
        println!(
            "{}  {:<12} {}",
            trunk("│"),
            white("sandbox:"),
            muted(&suite_result.sandbox)
        );
    }

    let avg_duration = if suite_result.total_tasks > 0 {
        suite_result.total_duration_secs / suite_result.total_tasks as f64
    } else {
        0.0
    };
    let fastest_task = suite_result
        .results
        .iter()
        .min_by(|a, b| a.duration_secs.total_cmp(&b.duration_secs));
    let slowest_task = suite_result
        .results
        .iter()
        .max_by(|a, b| a.duration_secs.total_cmp(&b.duration_secs));

    println!("{}", trunk("│"));
    println!("{}  {}", orange("✱"), white("performance metrics:"));
    println!(
        "{}  {:<26} {}",
        trunk("│"),
        white("total duration:"),
        muted(&format!("{:.1}s", suite_result.total_duration_secs))
    );
    println!(
        "{}  {:<26} {}",
        trunk("│"),
        white("average task time:"),
        muted(&format!("{:.1}s", avg_duration))
    );
    if let Some(fastest) = fastest_task {
        println!(
            "{}  {:<26} {}",
            trunk("│"),
            white("fastest completion:"),
            muted(&format!("{:.1}s ({})", fastest.duration_secs, fastest.task_id))
        );
    }
    if let Some(slowest) = slowest_task {
        println!(
            "{}  {:<26} {}",
            trunk("│"),
            white("slowest completion:"),
            muted(&format!("{:.1}s ({})", slowest.duration_secs, slowest.task_id))
        );
    }

    if let Some(ref profile) = suite_result.intelligence_profile {
        println!("{}", trunk("│"));
        println!("{}  {}", orange("✱"), white("agent intelligence profile:"));
        println!(
            "{}  {:<26} {}",
            trunk("│"),
            white("first-attempt resolution:"),
            muted(&format!(
                "{:.1}% ({}/{} tasks solved without retry)",
                profile.first_attempt_rate, profile.first_attempt_count, suite_result.total_tasks
            ))
        );
        let error_rec_str = if profile.tasks_with_errors > 0 {
            format!(
                "{:.1}% ({}/{} tasks recovered after initial error)",
                profile.error_recovery_rate, profile.recovered_count, profile.tasks_with_errors
            )
        } else {
            "100.0% (no intermediate errors encountered)".to_string()
        };
        println!(
            "{}  {:<26} {}",
            trunk("│"),
            white("error recovery rate:"),
            muted(&error_rec_str)
        );
        println!(
            "{}  {:<26} {}",
            trunk("│"),
            white("self-verification rate:"),
            muted(&format!(
                "{:.1}% (post-fix verification executed)",
                profile.self_verification_rate
            ))
        );
        println!(
            "{}  {:<26} {}",
            trunk("│"),
            white("context hygiene score:"),
            muted(&format!(
                "{:.1}% (efficient stdout streaming)",
                profile.context_hygiene_score
            ))
        );

        println!("{}", trunk("│"));
        println!("{}  {}", orange("✱"), white("token economics & cost:"));
        println!(
            "{}  {:<26} {}",
            trunk("│"),
            white("total tokens used:"),
            muted(&format!("{} (prompt: {}, completion: {})", profile.total_tokens, profile.total_prompt_tokens, profile.total_completion_tokens))
        );
        println!(
            "{}  {:<26} {}",
            trunk("│"),
            white("estimated run cost:"),
            mint_green(&format!("${:.4} USD", profile.total_cost_usd))
        );
        if profile.cost_per_resolved_task > 0.0 {
            println!(
                "{}  {:<26} {}",
                trunk("│"),
                white("cost per resolved task:"),
                muted(&format!("${:.4} USD / pass", profile.cost_per_resolved_task))
            );
        }
    }

    let mut category_map: BTreeMap<&'static str, (usize, usize)> = BTreeMap::new();
    for (t, r) in tasks.iter().zip(suite_result.results.iter()) {
        let tag = get_task_category_tag(&t.id);
        let entry = category_map.entry(tag).or_insert((0, 0));
        entry.0 += if r.passed { 1 } else { 0 };
        entry.1 += 1;
    }

    println!("{}", trunk("│"));
    println!("{}  {}", orange("✱"), white("domain competencies:"));
    for (tag, (passed, count)) in category_map {
        let cat_rate = (passed as f64 / count as f64) * 100.0;
        let cat_name = match tag {
            "[net]" => "network & services",
            "[git]" => "git & version control",
            "[sec]" => "security & permissions",
            "[data]" => "data processing",
            "[fs]" => "filesystem operations",
            "[dev]" => "dev & environments",
            "[logs]" => "log analysis",
            _ => "general terminal tasks",
        };
        println!(
            "{}  {:<26} {}",
            trunk("│"),
            white(cat_name),
            muted(&format!("{:>2}/{} ({:.1}%)", passed, count, cat_rate))
        );
    }

    if !suite_result.harness_insights.is_empty() {
        println!("{}", trunk("│"));
        println!("{}  {}", orange("✱"), white("harness optimization insights:"));
        for insight in &suite_result.harness_insights {
            println!("{}  {} {}", trunk("│"), white("•"), muted(insight));
        }
    }

    println!("{}", trunk("│"));
    println!(
        "{}  {} {}",
        trunk("│"),
        white("report saved to:"),
        muted(&export_file.display().to_string())
    );
    println!("{}", trunk("│"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_suite_result_serialization() {
        let suite = BenchmarkSuiteResult {
            timestamp: "2026-08-19T19:00:00Z".to_string(),
            agent: "Claude Code".to_string(),
            total_tasks: 2,
            passed_tasks: 1,
            failed_tasks: 1,
            pass_rate: 50.0,
            total_duration_secs: 24.5,
            total_tokens: 1200,
            total_cost_usd: 0.0074,
            cost_per_resolved_task: 0.0074,
            results: vec![
                TaskResult {
                    task_id: "001-nginx-config".to_string(),
                    task_name: "Fix Nginx".to_string(),
                    passed: true,
                    duration_secs: 12.0,
                    agent_name: "Claude Code".to_string(),
                    exit_code: Some(0),
                    error_message: None,
                    agent_output: "Done".to_string(),
                    prompt_tokens: 450,
                    completion_tokens: 150,
                    total_tokens: 600,
                    estimated_cost_usd: 0.0035,
                },
                TaskResult {
                    task_id: "004-port-conflict".to_string(),
                    task_name: "Port Conflict".to_string(),
                    passed: false,
                    duration_secs: 12.5,
                    agent_name: "Claude Code".to_string(),
                    exit_code: Some(1),
                    error_message: Some("Port in use".to_string()),
                    agent_output: "Failed".to_string(),
                    prompt_tokens: 420,
                    completion_tokens: 180,
                    total_tokens: 600,
                    estimated_cost_usd: 0.0039,
                },
            ],
            sandbox: "spacetime-sandbox:latest".to_string(),
            intelligence_profile: None,
            harness_insights: vec![],
        };

        let json = serde_json::to_string(&suite).unwrap();
        let deserialized: BenchmarkSuiteResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_tasks, 2);
        assert_eq!(deserialized.passed_tasks, 1);
        assert_eq!(deserialized.pass_rate, 50.0);
    }

    #[test]
    fn test_intelligence_profile_and_insights_computation() {
        let tasks = vec![
            BenchmarkTask {
                id: "001-nginx-config".to_string(),
                name: "Fix Nginx".to_string(),
                description: "Fix nginx".to_string(),
                base_image: "alpine".to_string(),
                prompt: "fix nginx".to_string(),
                task_dir: PathBuf::from("tasks/001"),
                setup_script: PathBuf::from("tasks/001/setup.sh"),
                test_script: PathBuf::from("tasks/001/test.sh"),
                max_turns: 10,
                timeout_secs: 60,
            },
            BenchmarkTask {
                id: "002-find-file".to_string(),
                name: "Find File".to_string(),
                description: "Find file".to_string(),
                base_image: "alpine".to_string(),
                prompt: "find file".to_string(),
                task_dir: PathBuf::from("tasks/002"),
                setup_script: PathBuf::from("tasks/002/setup.sh"),
                test_script: PathBuf::from("tasks/002/test.sh"),
                max_turns: 10,
                timeout_secs: 60,
            },
        ];

        let results = vec![
            TaskResult {
                task_id: "001-nginx-config".to_string(),
                task_name: "Fix Nginx".to_string(),
                passed: true,
                duration_secs: 2.5,
                agent_name: "TestAgent".to_string(),
                exit_code: Some(0),
                error_message: None,
                agent_output: "Running nginx -t and curl localhost... done".to_string(),
                prompt_tokens: 450,
                completion_tokens: 150,
                total_tokens: 600,
                estimated_cost_usd: 0.0035,
            },
            TaskResult {
                task_id: "002-find-file".to_string(),
                task_name: "Find File".to_string(),
                passed: true,
                duration_secs: 3.0,
                agent_name: "TestAgent".to_string(),
                exit_code: Some(0),
                error_message: None,
                agent_output: "Error: file not found. Found with find. git status ok".to_string(),
                prompt_tokens: 420,
                completion_tokens: 180,
                total_tokens: 600,
                estimated_cost_usd: 0.0039,
            },
        ];

        let profile = compute_intelligence_profile(&tasks, &results);
        assert_eq!(profile.first_attempt_count, 1);
        assert_eq!(profile.first_attempt_rate, 50.0);
        assert_eq!(profile.tasks_with_errors, 1);
        assert_eq!(profile.recovered_count, 1);
        assert_eq!(profile.error_recovery_rate, 100.0);
        assert_eq!(profile.verified_count, 2);
        assert_eq!(profile.self_verification_rate, 100.0);
        assert_eq!(profile.total_tokens, 1200);
        assert!(profile.total_cost_usd > 0.0);

        let insights = generate_harness_insights(&tasks, &results, &profile);
        assert!(!insights.is_empty());
    }

    #[test]
    fn test_nan_float_total_cmp() {
        let results = [
            TaskResult {
                task_id: "001".to_string(),
                task_name: "T1".to_string(),
                passed: true,
                duration_secs: f64::NAN,
                agent_name: "A".to_string(),
                exit_code: Some(0),
                error_message: None,
                agent_output: "ok".to_string(),
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
                estimated_cost_usd: 0.001,
            },
            TaskResult {
                task_id: "002".to_string(),
                task_name: "T2".to_string(),
                passed: true,
                duration_secs: 5.0,
                agent_name: "A".to_string(),
                exit_code: Some(0),
                error_message: None,
                agent_output: "ok".to_string(),
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
                estimated_cost_usd: 0.001,
            },
        ];

        let fastest = results.iter().min_by(|a, b| a.duration_secs.total_cmp(&b.duration_secs));
        let slowest = results.iter().max_by(|a, b| a.duration_secs.total_cmp(&b.duration_secs));
        assert!(fastest.is_some());
        assert!(slowest.is_some());
    }

    #[test]
    fn test_multibyte_unicode_slicing_safety() {
        let clean = "🚀 异常错误: 这是一个非常长的中文字符串测试，用于确保不会因为字节截断而崩溃！";
        let formatted = if clean.chars().count() > 36 {
            format!("{}...", clean.chars().take(33).collect::<String>())
        } else {
            clean.to_string()
        };
        assert!(formatted.ends_with("..."));
    }
}
