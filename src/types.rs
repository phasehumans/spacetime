use std::path::PathBuf;
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuiteResult {
    pub timestamp: String,
    pub agent: String,
    #[serde(default)]
    pub sandbox: String,
    pub total_tasks: usize,
    pub passed_tasks: usize,
    pub failed_tasks: usize,
    pub pass_rate: f64,
    pub total_duration_secs: f64,
    pub results: Vec<TaskResult>,
    #[serde(default)]
    pub intelligence_profile: Option<IntelligenceProfile>,
    #[serde(default)]
    pub harness_insights: Vec<String>,
}
