use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTask {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_image: String,
    pub prompt: String,
    pub task_dir: PathBuf,
    pub setup_script: PathBuf,
    pub test_script: PathBuf,
    pub max_turns: u32,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub task_name: String,
    pub passed: bool,
    pub duration_secs: f64,
    pub agent_name: String,
    pub exit_code: Option<i32>,
    pub error_message: Option<String>,
    pub agent_output: String,
    #[serde(default)]
    pub prompt_tokens: usize,
    #[serde(default)]
    pub completion_tokens: usize,
    #[serde(default)]
    pub total_tokens: usize,
    #[serde(default)]
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntelligenceProfile {
    pub first_attempt_rate: f64,
    pub first_attempt_count: usize,
    pub error_recovery_rate: f64,
    pub recovered_count: usize,
    pub tasks_with_errors: usize,
    pub self_verification_rate: f64,
    pub verified_count: usize,
    pub context_hygiene_score: f64,
    #[serde(default)]
    pub total_prompt_tokens: usize,
    #[serde(default)]
    pub total_completion_tokens: usize,
    #[serde(default)]
    pub total_tokens: usize,
    #[serde(default)]
    pub total_cost_usd: f64,
    #[serde(default)]
    pub cost_per_resolved_task: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkSuiteResult {
    pub timestamp: String,
    pub agent: String,
    /// Harness name without the model, so leaderboard rows can be keyed on
    /// agent + model instead of parsing the composite `agent` label.
    #[serde(default)]
    pub harness: String,
    #[serde(default)]
    pub model: Option<String>,
    /// CLI version that produced the report; submissions from different
    /// versions are not assumed to have run the suite identically.
    #[serde(default)]
    pub spacetime_version: String,
    /// Suite identity, see `taskset`. Empty for reports written before 0.0.4.
    #[serde(default)]
    pub taskset_id: String,
    #[serde(default)]
    pub taskset_hash: String,
    #[serde(default)]
    pub taskset_size: usize,
    #[serde(default)]
    pub sandbox: String,
    pub total_tasks: usize,
    pub passed_tasks: usize,
    pub failed_tasks: usize,
    pub pass_rate: f64,
    pub total_duration_secs: f64,
    #[serde(default)]
    pub total_tokens: usize,
    #[serde(default)]
    pub total_cost_usd: f64,
    #[serde(default)]
    pub cost_per_resolved_task: f64,
    pub results: Vec<TaskResult>,
    #[serde(default)]
    pub intelligence_profile: Option<IntelligenceProfile>,
    #[serde(default)]
    pub harness_insights: Vec<String>,
}
