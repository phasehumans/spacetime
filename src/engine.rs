use crate::config::AppConfig;
use crate::provider::{LlmProvider, Message};
use crate::sandbox::SandboxRuntime;
use crate::task::BenchmarkTask;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvaluationScorecard {
    pub task_id: String,
    pub provider: String,
    pub model: String,
    pub passed: bool,
    pub turns_used: usize,
    pub max_turns: usize,
    pub duration_seconds: u64,
    pub commands_executed: usize,
    pub validation_output: String,
    pub logs: Vec<String>,
    pub reasoning_history: Vec<String>,
}

pub trait EvaluationObserver: Send + Sync {
    fn on_turn_start(&mut self, _turn: usize) {}
    fn on_reasoning(&mut self, _turn: usize, _reasoning: &str) {}
    fn on_command(
        &mut self,
        _turn: usize,
        _command: &str,
        _stdout: &str,
        _stderr: &str,
        _exit_code: i64,
    ) {
    }
}

pub struct EvaluationEngine;

impl EvaluationEngine {
    pub async fn run_evaluation(
        task: &BenchmarkTask,
        provider: &dyn LlmProvider,
        runtime: &SandboxRuntime,
        config: &AppConfig,
        mut observer: Option<&mut dyn EvaluationObserver>,
    ) -> Result<EvaluationScorecard> {
        let start_time = Instant::now();
        let mut logs = Vec::new();
        let mut reasoning_history = Vec::new();
        let mut commands_executed = 0;
        let timeout_dur = Duration::from_secs(task.timeout_seconds);

        // 1. Create sandbox
        let mut sandbox = tokio::time::timeout(timeout_dur, runtime.create_sandbox(&task.base_image))
            .await
            .map_err(|_| anyhow!("Sandbox creation timed out after {}s", task.timeout_seconds))??;

        // 2. Run setup script if present
        if !task.setup_script.is_empty() {
            let setup_res = tokio::time::timeout(timeout_dur, sandbox.execute(&task.setup_script))
                .await
                .map_err(|_| anyhow!("Setup script execution timed out"))??;

            logs.push(format!("SETUP STDOUT: {}", setup_res.stdout));
            if !setup_res.stderr.is_empty() {
                logs.push(format!("SETUP STDERR: {}", setup_res.stderr));
            }
        }

        // 3. Prepare initial turn messages
        let mut messages = vec![Message {
            role: "user".to_string(),
            content: format!("Task Objective: {}\nDescription: {}", task.prompt, task.description),
        }];

        let mut turns_used = 0;

        // 4. Execution loop
        while turns_used < task.max_turns {
            turns_used += 1;
            if let Some(ref mut obs) = observer {
                obs.on_turn_start(turns_used);
            }

            let agent_resp = tokio::time::timeout(timeout_dur, provider.chat(&messages))
                .await
                .map_err(|_| anyhow!("LLM Provider response timed out on turn {}", turns_used))??;

            reasoning_history.push(agent_resp.reasoning.clone());
            if let Some(ref mut obs) = observer {
                obs.on_reasoning(turns_used, &agent_resp.reasoning);
            }

            match agent_resp.command {
                Some(ref cmd) if !cmd.trim().is_empty() => {
                    commands_executed += 1;
                    let exec_res = tokio::time::timeout(timeout_dur, sandbox.execute(cmd))
                        .await
                        .map_err(|_| anyhow!("Command execution timed out on turn {}", turns_used))??;

                    let log_entry = format!(
                        "[$ {}] exit: {}\nstdout:\n{}\nstderr:\n{}",
                        cmd, exec_res.exit_code, exec_res.stdout, exec_res.stderr
                    );
                    logs.push(log_entry.clone());

                    if let Some(ref mut obs) = observer {
                        obs.on_command(
                            turns_used,
                            cmd,
                            &exec_res.stdout,
                            &exec_res.stderr,
                            exec_res.exit_code,
                        );
                    }

                    // Feed observation back to agent
                    messages.push(Message {
                        role: "assistant".to_string(),
                        content: serde_json::to_string(&agent_resp)?,
                    });
                    messages.push(Message {
                        role: "user".to_string(),
                        content: format!(
                            "Command Output (exit code {}):\nSTDOUT:\n{}\nSTDERR:\n{}",
                            exec_res.exit_code, exec_res.stdout, exec_res.stderr
                        ),
                    });
                }
                _ => {
                    // Agent finished task
                    break;
                }
            }
        }

        // 5. Run validation script
        let val_res = tokio::time::timeout(timeout_dur, sandbox.execute(&task.validation_script))
            .await
            .map_err(|_| anyhow!("Validation script execution timed out"))??;

        let passed = val_res.exit_code == 0;
        let validation_output = format!(
            "Exit Code: {}\nSTDOUT: {}\nSTDERR: {}",
            val_res.exit_code, val_res.stdout, val_res.stderr
        );

        // 6. Explicit sandbox teardown
        sandbox.destroy().await?;

        let duration_seconds = start_time.elapsed().as_secs();

        Ok(EvaluationScorecard {
            task_id: task.id.clone(),
            provider: config.provider.clone(),
            model: config.model.clone(),
            passed,
            turns_used,
            max_turns: task.max_turns,
            duration_seconds,
            commands_executed,
            validation_output,
            logs,
            reasoning_history,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scorecard_serialization() {
        let scorecard = EvaluationScorecard {
            task_id: "task-001".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            passed: true,
            turns_used: 3,
            max_turns: 15,
            duration_seconds: 12,
            commands_executed: 2,
            validation_output: "Welcome to nginx!".to_string(),
            logs: vec!["apt-get update".to_string()],
            reasoning_history: vec!["Inspect nginx config".to_string()],
        };

        let json = serde_json::to_string(&scorecard).unwrap();
        assert!(json.contains("task-001"));
        assert!(json.contains("gpt-4o"));
    }
}
