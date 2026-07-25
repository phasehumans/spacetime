use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkTask {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_image: String,
    pub prompt: String,
    pub setup_script: String,
    pub validation_script: String,
    pub max_turns: usize,
    pub timeout_seconds: u64,
}

impl BenchmarkTask {
    pub fn parse_bash_script(content: &str) -> Result<Self> {
        let setup_marker = "# === SETUP ===";
        let validate_marker = "# === VALIDATE ===";

        let setup_idx = content.find(setup_marker);
        let validate_idx = content.find(validate_marker);

        let validate_pos = validate_idx.ok_or_else(|| {
            anyhow!("Missing required '# === VALIDATE ===' section marker in task script")
        })?;

        let header_part = match setup_idx {
            Some(idx) => &content[..idx],
            None => &content[..validate_pos],
        };

        let setup_script = match setup_idx {
            Some(s_pos) => {
                let start = s_pos + setup_marker.len();
                content[start..validate_pos].trim().to_string()
            }
            None => String::new(),
        };

        let validation_script = content[validate_pos + validate_marker.len()..]
            .trim()
            .to_string();

        let mut id = String::new();
        let mut name = String::new();
        let mut description = String::new();
        let mut base_image = "ubuntu:22.04".to_string();
        let mut prompt = String::new();
        let mut max_turns = 15;
        let mut timeout_seconds = 300;

        for line in header_part.lines() {
            let trimmed = line.trim();
            if let Some(comment) = trimmed.strip_prefix('#') {
                let comment = comment.trim();
                if let Some((key, val)) = comment.split_once(':') {
                    let key = key.trim();
                    let val = val.trim();
                    match key {
                        "id" => id = val.to_string(),
                        "name" => name = val.to_string(),
                        "description" => description = val.to_string(),
                        "base_image" => base_image = val.to_string(),
                        "prompt" => prompt = val.to_string(),
                        "max_turns" => {
                            if let Ok(v) = val.parse::<usize>() {
                                max_turns = v;
                            }
                        }
                        "timeout" | "timeout_seconds" => {
                            if let Ok(v) = val.parse::<u64>() {
                                timeout_seconds = v;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if id.is_empty() {
            return Err(anyhow!("Task metadata is missing required 'id' header"));
        }

        Ok(BenchmarkTask {
            id,
            name,
            description,
            base_image,
            prompt,
            setup_script,
            validation_script,
            max_turns,
            timeout_seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_bash_task() {
        let script = r#"#!/usr/bin/env spacetime
# id: task-001
# name: Fix Nginx Configuration
# description: Fix syntax error in nginx config
# base_image: ubuntu:22.04
# max_turns: 15
# timeout: 300
# prompt: The Nginx server is failing to start due to a syntax error. Fix it.

# === SETUP ===
apt-get update && apt-get install -y nginx curl
echo "server { listen 80; }" > /etc/nginx/sites-available/default

# === VALIDATE ===
nginx -t
service nginx start
curl -s http://localhost
"#;

        let task = BenchmarkTask::parse_bash_script(script).unwrap();
        assert_eq!(task.id, "task-001");
        assert_eq!(task.name, "Fix Nginx Configuration");
        assert_eq!(task.description, "Fix syntax error in nginx config");
        assert_eq!(task.base_image, "ubuntu:22.04");
        assert_eq!(task.max_turns, 15);
        assert_eq!(task.timeout_seconds, 300);
        assert_eq!(task.prompt, "The Nginx server is failing to start due to a syntax error. Fix it.");
        assert!(task.setup_script.contains("apt-get update"));
        assert!(task.validation_script.contains("nginx -t"));
    }

    #[test]
    fn test_parse_task_missing_validate_fails() {
        let script = r#"# id: task-002
# name: Test
# prompt: Do something

# === SETUP ===
echo "hello"
"#;
        assert!(BenchmarkTask::parse_bash_script(script).is_err());
    }
}
