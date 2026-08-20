use std::path::Path;
use std::time::Duration;
use anyhow::{Context, Result, anyhow};
use bollard::Docker;
use bollard::container::{
    Config, CreateContainerOptions, LogOutput, RemoveContainerOptions, StartContainerOptions,
    StopContainerOptions,
};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::service::HostConfig;
use colored::*;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

use crate::types::ExecutionResult;

pub const DEFAULT_SANDBOX_IMAGE: &str = "spacetime-sandbox:latest";

pub async fn ensure_sandbox_image(image_tag: &str, force_rebuild: bool) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()
        .with_context(|| "Failed to connect to local Docker daemon. Is Docker running?")?;

    let image_exists = docker.inspect_image(image_tag).await.is_ok();

    if image_exists && !force_rebuild {
        return Ok(());
    }

    println!(
        "{}",
        format!("⚙ Sandbox image '{}' not found or rebuild requested. Building...", image_tag).yellow()
    );

    let dockerfile = "Dockerfile.sandbox";
    if !Path::new(dockerfile).exists() {
        return Err(anyhow!(
            "Dockerfile.sandbox not found in current directory. Cannot build sandbox."
        ));
    }

    let spinner = create_spinner("Building Docker sandbox image (this may take ~1-2 min on first run)...");

    let status = TokioCommand::new("docker")
        .arg("build")
        .arg("-t")
        .arg(image_tag)
        .arg("-f")
        .arg(dockerfile)
        .arg(".")
        .status()
        .await
        .with_context(|| "Failed to execute 'docker build'. Is Docker installed and in PATH?")?;

    spinner.finish_and_clear();

    if !status.success() {
        return Err(anyhow!(
            "Failed to build sandbox image '{}'. Docker build exited with status: {}",
            image_tag,
            status
        ));
    }

    println!(
        "{}",
        format!("✓ Sandbox image '{}' built successfully!", image_tag).green()
    );

    Ok(())
}

pub struct EnvironmentManager {
    docker: Docker,
    container_id: Option<String>,
    image: String,
    mount_dir: Option<String>,
}

impl EnvironmentManager {
    #[allow(dead_code)]
    pub fn new(image: String) -> Result<Self> {
        Self::with_mount(image, None)
    }

    pub fn with_mount(image: String, mount_dir: Option<String>) -> Result<Self> {
        let docker = Docker::connect_with_local_defaults()
            .with_context(|| "Failed to connect to local Docker daemon. Is Docker running?")?;
        Ok(Self {
            docker,
            container_id: None,
            image,
            mount_dir,
        })
    }

    #[allow(dead_code)]
    pub fn get_container_id(&self) -> Option<&str> {
        self.container_id.as_deref()
    }

    pub async fn initialize(&mut self) -> Result<()> {
        let container_name = format!("spacetime-{}", uuid_simple());

        let options = Some(CreateContainerOptions::<String> {
            name: container_name,
            platform: None,
        });

        let host_config = self.mount_dir.as_ref().map(|dir| HostConfig {
            binds: Some(vec![format!("{}:/workspace:ro", dir)]),
            ..Default::default()
        });

        let config = Config {
            image: Some(self.image.clone()),
            cmd: Some(vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "sleep infinity".to_string(),
            ]),
            tty: Some(true),
            working_dir: Some("/root".to_string()),
            host_config,
            ..Default::default()
        };

        let container = self
            .docker
            .create_container(options, config)
            .await
            .with_context(|| format!("Failed to create Docker container with image '{}'", self.image))?;

        self.container_id = Some(container.id.clone());

        self.docker
            .start_container(&container.id, None::<StartContainerOptions<String>>)
            .await
            .with_context(|| format!("Failed to start Docker container '{}'", container.id))?;

        Ok(())
    }

    pub async fn execute_host_script(&self, host_script_path: &Path, timeout_secs: u64) -> Result<ExecutionResult> {
        let container_id = self
            .container_id
            .as_ref()
            .ok_or_else(|| anyhow!("Container is not initialized"))?;

        let script_content = std::fs::read_to_string(host_script_path)
            .with_context(|| format!("Failed to read script from {}", host_script_path.display()))?;

        let start_time = std::time::Instant::now();

        let exec_config = CreateExecOptions {
            cmd: Some(vec![
                "/bin/bash".to_string(),
                "-c".to_string(),
                script_content,
            ]),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec = self
            .docker
            .create_exec(container_id, exec_config)
            .await
            .with_context(|| "Failed to create exec instance for host script")?;

        let start_results = self.docker.start_exec(&exec.id, None).await?;

        let mut stdout_raw = String::new();
        let mut stderr_raw = String::new();
        let mut timed_out = false;

        if let StartExecResults::Attached { mut output, .. } = start_results {
            let timeout_duration = Duration::from_secs(timeout_secs);
            let read_future = async {
                while let Some(msg) = output.next().await {
                    match msg {
                        Ok(LogOutput::StdOut { message }) => {
                            stdout_raw.push_str(&String::from_utf8_lossy(&message));
                        }
                        Ok(LogOutput::StdErr { message }) => {
                            stderr_raw.push_str(&String::from_utf8_lossy(&message));
                        }
                        Ok(LogOutput::Console { message }) => {
                            stdout_raw.push_str(&String::from_utf8_lossy(&message));
                        }
                        _ => {}
                    }
                }
            };

            if timeout(timeout_duration, read_future).await.is_err() {
                timed_out = true;
                stderr_raw.push_str(&format!(
                    "\n[Spacetime Warning] Script execution timed out ({}s).\n",
                    timeout_secs
                ));
            }
        }

        let exit_code = if timed_out {
            124
        } else {
            let inspect = self.docker.inspect_exec(&exec.id).await?;
            inspect.exit_code.unwrap_or(0) as i32
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(ExecutionResult {
            stdout: stdout_raw.trim().to_string(),
            stderr: stderr_raw.trim().to_string(),
            exit_code,
            timed_out,
            duration_ms,
        })
    }

    pub async fn execute_agent_stream(
        &self,
        command: &str,
        env_vars: &[String],
        timeout_secs: u64,
        silent: bool,
    ) -> Result<ExecutionResult> {
        let container_id = self
            .container_id
            .as_ref()
            .ok_or_else(|| anyhow!("Container is not initialized"))?;

        let start_time = std::time::Instant::now();

        let exec_config = CreateExecOptions {
            cmd: Some(vec!["/bin/bash".to_string(), "-c".to_string(), command.to_string()]),
            env: Some(env_vars.to_vec()),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            tty: Some(true),
            working_dir: Some("/root".to_string()),
            ..Default::default()
        };

        let exec = self
            .docker
            .create_exec(container_id, exec_config)
            .await
            .with_context(|| "Failed to create exec instance for agent")?;

        let start_results = self.docker.start_exec(&exec.id, None).await?;

        let mut output_buffer = String::new();
        let mut timed_out = false;

        if let StartExecResults::Attached { mut output, .. } = start_results {
            let timeout_duration = Duration::from_secs(timeout_secs);
            let read_future = async {
                while let Some(msg) = output.next().await {
                    match msg {
                        Ok(LogOutput::StdOut { message })
                        | Ok(LogOutput::StdErr { message })
                        | Ok(LogOutput::Console { message }) => {
                            let text = String::from_utf8_lossy(&message);
                            if !silent {
                                print!("{}", text);
                                use std::io::Write;
                                let _ = std::io::stdout().flush();
                            }
                            output_buffer.push_str(&text);
                        }
                        _ => {}
                    }
                }
            };

            if timeout(timeout_duration, read_future).await.is_err() {
                timed_out = true;
                let warn_msg = format!(
                    "\n\n[Spacetime Warning] Agent execution exceeded timeout limit of {}s.\n",
                    timeout_secs
                );
                if !silent {
                    eprintln!("{}", warn_msg.bright_red());
                }
                output_buffer.push_str(&warn_msg);
            }
        }

        let exit_code = if timed_out {
            124
        } else {
            let inspect = self.docker.inspect_exec(&exec.id).await?;
            inspect.exit_code.unwrap_or(0) as i32
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(ExecutionResult {
            stdout: output_buffer.clone(),
            stderr: String::new(),
            exit_code,
            timed_out,
            duration_ms,
        })
    }

    pub async fn destroy(&mut self) -> Result<()> {
        if let Some(container_id) = self.container_id.take() {
            let _ = self
                .docker
                .stop_container(&container_id, Some(StopContainerOptions { t: 1 }))
                .await;
            let _ = self
                .docker
                .remove_container(
                    &container_id,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await;
        }
        Ok(())
    }
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

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", nanos)
}
