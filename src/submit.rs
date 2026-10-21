//! Leaderboard submission and terminal rendering.
//!
//! Results are submitted as files in the repository rather than posted to an
//! API. That keeps the trust boundary in one place: a submission is inert data
//! until CI validates it, and a human merges it. See `scripts/leaderboard-lib.mjs`
//! for the same checks implemented in JavaScript.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::tui::theme::{coral_red, mint_green, muted, orange, trunk, white};
use crate::types::BenchmarkSuiteResult;

pub const SUBMISSION_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_SUBMISSIONS_DIR: &str = "leaderboard/submissions";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// Self-reported by the submitter. Rendered with a badge, never mixed
    /// into the verified ranking without that badge.
    Community,
    /// Re-run by a maintainer or CI on a pinned commit, image and taskset.
    Verified,
}

impl Tier {
    pub fn parse(value: &str) -> Result<Tier> {
        match value.trim().to_lowercase().as_str() {
            "community" => Ok(Tier::Community),
            "verified" => Ok(Tier::Verified),
            other => bail!("unknown tier '{}' (expected community or verified)", other),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Tier::Community => "community",
            Tier::Verified => "verified",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub schema_version: u32,
    pub submitted_by: String,
    pub tier: Tier,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    pub run: BenchmarkSuiteResult,
}

/// Recomputed counts. A submission's denormalized totals are never trusted:
/// they must agree with the per-task results it ships.
pub struct Counts {
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f64,
}

pub fn recount(run: &BenchmarkSuiteResult) -> Counts {
    let total = run.results.len();
    let passed = run.results.iter().filter(|r| r.passed).count();
    let pass_rate = if total == 0 {
        0.0
    } else {
        (passed as f64 / total as f64) * 100.0
    };
    Counts {
        total,
        passed,
        pass_rate,
    }
}

/// Returns every reason the report cannot be accepted as a submission, so the
/// user sees them all at once instead of fixing one per run.
pub fn validate_report(run: &BenchmarkSuiteResult) -> Vec<String> {
    let mut problems = Vec::new();

    if run.taskset_id.is_empty() || run.taskset_hash.is_empty() {
        problems.push(
            "report has no taskset identity; rerun the suite with this version of spacetime"
                .to_string(),
        );
    }
    if run.results.is_empty() {
        problems.push("report contains no task results".to_string());
        return problems;
    }

    let counts = recount(run);
    if counts.total != run.total_tasks {
        problems.push(format!(
            "total_tasks says {} but {} task results are present",
            run.total_tasks, counts.total
        ));
    }
    if counts.passed != run.passed_tasks {
        problems.push(format!(
            "passed_tasks says {} but {} results are marked passed",
            run.passed_tasks, counts.passed
        ));
    }
    if (counts.pass_rate - run.pass_rate).abs() > 0.01 {
        problems.push(format!(
            "pass_rate says {:.2}% but the results give {:.2}%",
            run.pass_rate, counts.pass_rate
        ));
    }

    let mut seen = std::collections::HashSet::new();
    for result in &run.results {
        if !seen.insert(result.task_id.clone()) {
            problems.push(format!("task {} appears more than once", result.task_id));
        }
    }

    if run.harness.trim().is_empty() {
        problems.push("report has no harness name".to_string());
    }

    problems
}

fn slug(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn submission_stem(run: &BenchmarkSuiteResult) -> String {
    let harness = slug(&run.harness);
    let model = run
        .model
        .as_deref()
        .map(slug)
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| "default".to_string());
    format!("{}__{}", harness, model)
}

fn run_metrics(run: &BenchmarkSuiteResult) -> (f64, f64) {
    let counts = recount(run);
    let avg_secs = if counts.total == 0 {
        0.0
    } else {
        run.total_duration_secs / counts.total as f64
    };
    (counts.pass_rate, avg_secs)
}

pub struct SubmitOptions {
    pub report: PathBuf,
    pub submissions_dir: PathBuf,
    pub submitted_by: String,
    pub tier: Tier,
    pub source_url: Option<String>,
    pub notes: Option<String>,
}

pub fn submit(options: SubmitOptions) -> Result<PathBuf> {
    let raw = fs::read_to_string(&options.report)
        .with_context(|| format!("Failed to read report at {}", options.report.display()))?;
    let run: BenchmarkSuiteResult = serde_json::from_str(&raw)
        .with_context(|| format!("{} is not a spacetime report", options.report.display()))?;

    let problems = validate_report(&run);
    if !problems.is_empty() {
        let mut message = format!("{} cannot be submitted:", options.report.display());
        for problem in &problems {
            message.push_str(&format!("\n  - {}", problem));
        }
        bail!(message);
    }

    if options.submitted_by.trim().is_empty() {
        bail!("a submitter handle is required (use --user <github-handle>)");
    }

    let submission = Submission {
        schema_version: SUBMISSION_SCHEMA_VERSION,
        submitted_by: options
            .submitted_by
            .trim()
            .trim_start_matches('@')
            .to_string(),
        tier: options.tier,
        source_url: options.source_url.clone(),
        notes: options.notes.clone(),
        run,
    };

    let target_dir = options.submissions_dir.join(&submission.run.taskset_id);
    fs::create_dir_all(&target_dir).with_context(|| {
        format!(
            "Failed to create submissions directory at {}",
            target_dir.display()
        )
    })?;

    let target = target_dir.join(format!("{}.json", submission_stem(&submission.run)));
    if target.exists() {
        bail!(
            "{} already has a submission for this agent, model and taskset.\n\
             Bump the run (or delete the old file) if this is a re-run you want to replace.",
            target.display()
        );
    }

    let json = serde_json::to_string_pretty(&submission)?;
    fs::write(&target, format!("{}\n", json))
        .with_context(|| format!("Failed to write {}", target.display()))?;

    let counts = recount(&submission.run);
    println!();
    println!("{}  {}", orange("✱"), white("submission written"));
    println!(
        "{}  {:<14} {}",
        trunk("│"),
        white("agent:"),
        muted(&format!(
            "{}{}",
            submission.run.harness,
            submission
                .run
                .model
                .as_deref()
                .map(|m| format!(" ({})", m))
                .unwrap_or_default()
        ))
    );
    println!(
        "{}  {:<14} {}",
        trunk("│"),
        white("taskset:"),
        muted(&submission.run.taskset_id)
    );
    println!(
        "{}  {:<14} {}",
        trunk("│"),
        white("score:"),
        mint_green(&format!(
            "{}/{} ({:.1}%)",
            counts.passed, counts.total, counts.pass_rate
        ))
    );
    println!(
        "{}  {:<14} {}",
        trunk("│"),
        white("tier:"),
        muted(submission.tier.label())
    );
    println!(
        "{}  {:<14} {}",
        trunk("│"),
        white("file:"),
        muted(&target.display().to_string())
    );
    println!("{}", trunk("│"));
    println!(
        "{}  {}",
        orange("✱"),
        white("open a pull request to have CI validate it:")
    );
    println!(
        "{}  {}",
        trunk("│"),
        muted(&format!(
            "git checkout -b leaderboard/{}",
            submission_stem(&submission.run)
        ))
    );
    println!(
        "{}  {}",
        trunk("│"),
        muted(&format!(
            "git add {} && git commit -m \"leaderboard: add run\"",
            target.display()
        ))
    );
    println!(
        "{}  {}",
        trunk("│"),
        muted("git push -u origin HEAD && gh pr create --fill")
    );
    println!("{}", trunk("│"));
    println!(
        "{}  {}",
        trunk("│"),
        muted("community runs are merged as self-reported; maintainers can re-run for verified")
    );
    println!();

    Ok(target)
}

/// A submission paired with the path it came from, for error reporting.
struct Loaded {
    path: PathBuf,
    submission: Submission,
}

fn load_submissions(dir: &Path) -> Result<Vec<Loaded>> {
    let mut loaded = Vec::new();
    if !dir.exists() {
        return Ok(loaded);
    }
    collect_json(dir, &mut loaded)?;
    Ok(loaded)
}

fn collect_json(dir: &Path, out: &mut Vec<Loaded>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_json(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let raw = fs::read_to_string(&path)?;
            let submission: Submission = serde_json::from_str(&raw)
                .with_context(|| format!("Failed to parse submission at {}", path.display()))?;
            out.push(Loaded { path, submission });
        }
    }
    Ok(())
}

fn sort_key(run: &BenchmarkSuiteResult) -> (std::cmp::Reverse<u64>, u64, u64) {
    let counts = recount(run);
    let (_, avg_secs) = run_metrics(run);
    // Rank by pass rate first, then by what a solved task costs, then speed.
    let pass_bp = (counts.pass_rate * 100.0).round() as u64;
    // An unknown cost is not a free cost: sort it after every known cost, the
    // same way `compareRows` does in scripts/leaderboard-lib.mjs.
    let cost_micros = if run.cost_per_resolved_task > 0.0 {
        (run.cost_per_resolved_task * 1_000_000.0).round() as u64
    } else {
        u64::MAX
    };
    let avg_ms = (avg_secs * 1000.0).round() as u64;
    (std::cmp::Reverse(pass_bp), cost_micros, avg_ms)
}

pub fn render_leaderboard(dir: &Path, as_json: bool, filter: Option<String>) -> Result<()> {
    let mut submissions = load_submissions(dir)?;

    if let Some(filter) = filter {
        submissions.retain(|s| s.submission.run.taskset_id == filter);
    }

    if as_json {
        let mut rows: Vec<_> = submissions
            .iter()
            .map(|s| {
                let counts = recount(&s.submission.run);
                serde_json::json!({
                    "agent": s.submission.run.harness,
                    "model": s.submission.run.model,
                    "tier": s.submission.tier,
                    "taskset_id": s.submission.run.taskset_id,
                    "passed": counts.passed,
                    "total": counts.total,
                    "pass_rate": counts.pass_rate,
                    "cost_per_resolved_task": s.submission.run.cost_per_resolved_task,
                    "avg_task_secs": run_metrics(&s.submission.run).1,
                    "submitted_by": s.submission.submitted_by,
                    "submission_file": s.path.display().to_string(),
                })
            })
            .collect();
        rows.sort_by(|a, b| {
            let ap = a["pass_rate"].as_f64().unwrap_or(0.0);
            let bp = b["pass_rate"].as_f64().unwrap_or(0.0);
            bp.partial_cmp(&ap).unwrap_or(std::cmp::Ordering::Equal)
        });
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }

    if submissions.is_empty() {
        println!();
        println!("{}  {}", orange("✱"), white("no submissions found"));
        println!(
            "{}  {:<10} {}",
            trunk("│"),
            white("looked in:"),
            muted(&dir.display().to_string())
        );
        println!(
            "{}  {}",
            trunk("│"),
            muted("run 'spacetime eval-all' then 'spacetime submit --report <file> --user <you>'")
        );
        println!();
        return Ok(());
    }

    // Group by taskset so runs produced against different task suites are never
    // ranked against each other.
    let mut by_taskset: std::collections::BTreeMap<String, Vec<&Loaded>> =
        std::collections::BTreeMap::new();
    for submission in &submissions {
        by_taskset
            .entry(submission.submission.run.taskset_id.clone())
            .or_default()
            .push(submission);
    }

    for (taskset_id, mut group) in by_taskset {
        group.sort_by_key(|entry| sort_key(&entry.submission.run));

        let size = group
            .first()
            .map(|s| recount(&s.submission.run).total)
            .unwrap_or(0);

        println!();
        println!(
            "{}  {}",
            orange("✱"),
            white(&format!("SPACETIME LEADERBOARD · {} tasks", size))
        );
        println!(
            "{}  {:<10} {}",
            trunk("│"),
            white("taskset:"),
            muted(&taskset_id)
        );
        println!("{}", trunk("│"));
        println!(
            "{}  {:<3} {:<26} {:<12} {:<10} {:<9} {}",
            trunk("│"),
            muted("#"),
            muted("agent"),
            muted("pass rate"),
            muted("$/solved"),
            muted("avg task"),
            muted("tier")
        );

        for (index, entry) in group.iter().enumerate() {
            let run = &entry.submission.run;
            let counts = recount(run);
            let (_, avg_secs) = run_metrics(run);
            let agent = format!(
                "{}{}",
                run.harness,
                run.model
                    .as_deref()
                    .map(|m| format!(" ({})", m))
                    .unwrap_or_default()
            );
            let agent = if agent.chars().count() > 25 {
                format!("{}…", agent.chars().take(24).collect::<String>())
            } else {
                agent
            };
            let cost = if run.cost_per_resolved_task > 0.0 {
                format!("${:.3}", run.cost_per_resolved_task)
            } else {
                "-".to_string()
            };
            let tier = match entry.submission.tier {
                Tier::Verified => mint_green("verified"),
                Tier::Community => coral_red("community"),
            };

            println!(
                "{}  {:<3} {:<26} {:<12} {:<10} {:<9} {}",
                trunk("│"),
                muted(&(index + 1).to_string()),
                white(&agent),
                mint_green(&format!("{:.1}%", counts.pass_rate)),
                muted(&cost),
                muted(&format!("{:.1}s", avg_secs)),
                tier
            );
        }

        println!("{}", trunk("│"));
        println!(
            "{}  {}",
            trunk("│"),
            muted("ranked by pass rate, then cost per resolved task, then average task time")
        );
        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaskResult;

    fn result(id: &str, passed: bool) -> TaskResult {
        TaskResult {
            task_id: id.to_string(),
            task_name: id.to_string(),
            passed,
            duration_secs: 10.0,
            agent_name: "Claude Code".to_string(),
            exit_code: Some(if passed { 0 } else { 1 }),
            error_message: None,
            agent_output: String::new(),
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            estimated_cost_usd: 0.0,
        }
    }

    fn report(results: Vec<TaskResult>) -> BenchmarkSuiteResult {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        BenchmarkSuiteResult {
            harness: "Claude Code".to_string(),
            taskset_id: "v1-2-1a2b3c4d".to_string(),
            taskset_hash: "abc".to_string(),
            taskset_size: total,
            total_tasks: total,
            passed_tasks: passed,
            failed_tasks: total - passed,
            pass_rate: (passed as f64 / total as f64) * 100.0,
            total_duration_secs: 20.0,
            results,
            ..Default::default()
        }
    }

    #[test]
    fn a_consistent_report_validates() {
        let run = report(vec![result("001", true), result("002", false)]);
        assert!(validate_report(&run).is_empty());
        assert_eq!(recount(&run).passed, 1);
    }

    #[test]
    fn edited_headline_numbers_are_rejected() {
        let mut run = report(vec![result("001", true), result("002", false)]);
        run.passed_tasks = 2;
        run.pass_rate = 100.0;

        let problems = validate_report(&run);
        assert!(problems.iter().any(|p| p.contains("passed_tasks")));
        assert!(problems.iter().any(|p| p.contains("pass_rate")));
    }

    #[test]
    fn duplicate_and_missing_taskset_identity_are_rejected() {
        let mut run = report(vec![result("001", true), result("001", true)]);
        run.taskset_id.clear();
        run.taskset_hash.clear();

        let problems = validate_report(&run);
        assert!(problems.iter().any(|p| p.contains("taskset identity")));
        assert!(problems.iter().any(|p| p.contains("more than once")));
    }

    #[test]
    fn submission_stem_is_stable_and_file_safe() {
        let mut run = report(vec![result("001", true)]);
        run.harness = "Claude Code".to_string();
        run.model = Some("claude-sonnet-4-5".to_string());
        assert_eq!(submission_stem(&run), "claude-code__claude-sonnet-4-5");

        run.model = None;
        assert_eq!(submission_stem(&run), "claude-code__default");
    }
}
