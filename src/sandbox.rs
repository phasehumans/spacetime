use anyhow::Result;
use bollard::container::{Config as BollardConfig, CreateContainerOptions, RemoveContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use bollard::secret::HostConfig;
use bollard::Docker;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i64,
}

pub struct SandboxGuard {
    pub container_id: String,
    pub docker: Docker,
    pub destroyed: bool,
}

impl SandboxGuard {
    pub async fn execute(&self, command: &str) -> Result<ExecutionResult> {
        let exec = self
            .docker
            .create_exec(
                &self.container_id,
                CreateExecOptions {
                    cmd: Some(vec!["/bin/sh", "-c", command]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await?;

        let start_results = self.docker.start_exec(&exec.id, None).await?;

        let mut stdout = String::new();
        let mut stderr = String::new();

        if let StartExecResults::Attached { mut output, .. } = start_results {
            while let Some(msg) = output.next().await {
                match msg? {
                    bollard::container::LogOutput::StdOut { message } => {
                        stdout.push_str(&String::from_utf8_lossy(&message));
                    }
                    bollard::container::LogOutput::StdErr { message } => {
                        stderr.push_str(&String::from_utf8_lossy(&message));
                    }
                    _ => {}
                }
            }
        }

        let inspect = self.docker.inspect_exec(&exec.id).await?;
        let exit_code = inspect.exit_code.unwrap_or(0);

        Ok(ExecutionResult {
            stdout: stdout.trim().to_string(),
            stderr: stderr.trim().to_string(),
            exit_code,
        })
    }

    pub async fn destroy(&mut self) -> Result<()> {
        if !self.destroyed {
            self.docker
                .remove_container(
                    &self.container_id,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await?;
            self.destroyed = true;
        }
        Ok(())
    }
}

impl Drop for SandboxGuard {
    fn drop(&mut self) {
        if !self.destroyed {
            let container_id = self.container_id.clone();
            let docker = self.docker.clone();
            tokio::spawn(async move {
                let _ = docker
                    .remove_container(
                        &container_id,
                        Some(RemoveContainerOptions {
                            force: true,
                            ..Default::default()
                        }),
                    )
                    .await;
            });
        }
    }
}

pub struct SandboxRuntime {
    docker: Docker,
}

impl SandboxRuntime {
    pub fn new() -> Result<Self> {
        let docker = Docker::connect_with_socket_defaults()?;
        Ok(Self { docker })
    }

    pub async fn create_sandbox(&self, image: &str) -> Result<SandboxGuard> {
        // Ensure image is present
        let mut stream = self.docker.create_image(
            Some(CreateImageOptions {
                from_image: image,
                ..Default::default()
            }),
            None,
            None,
        );

        while let Some(result) = stream.next().await {
            if let Err(e) = result {
                info!("Pulling image status warning: {}", e);
            }
        }

        let container_config = BollardConfig {
            image: Some(image.to_string()),
            cmd: Some(vec!["/bin/sh".to_string(), "-c".to_string(), "sleep infinity".to_string()]),
            tty: Some(true),
            host_config: Some(HostConfig {
                auto_remove: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };

        let container = self
            .docker
            .create_container(None::<CreateContainerOptions<String>>, container_config)
            .await?;

        self.docker
            .start_container::<String>(&container.id, None)
            .await?;

        Ok(SandboxGuard {
            container_id: container.id,
            docker: self.docker.clone(),
            destroyed: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sandbox_unit_creation() {
        let runtime = SandboxRuntime::new();
        assert!(runtime.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_sandbox_lifecycle_live_docker() {
        let runtime = SandboxRuntime::new().unwrap();
        let mut sandbox = runtime.create_sandbox("alpine:latest").await.unwrap();

        let result = sandbox.execute("echo 'hello spacetime'").await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello spacetime"));

        sandbox.destroy().await.unwrap();
        assert!(sandbox.destroyed);
    }
}
