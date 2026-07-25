use crate::task::BenchmarkTask;
use anyhow::{anyhow, Result};
use rust_embed::RustEmbed;
use std::fs;
use std::path::Path;

#[derive(RustEmbed)]
#[folder = "tasks/"]
pub struct EmbeddedTasks;

pub struct TaskLoader;

impl TaskLoader {
    pub fn load_embedded() -> Result<Vec<BenchmarkTask>> {
        let mut tasks = Vec::new();
        for file in EmbeddedTasks::iter() {
            if file.ends_with(".sh") || file.ends_with(".yaml") {
                if let Some(content_file) = EmbeddedTasks::get(file.as_ref()) {
                    let content_str = std::str::from_utf8(content_file.data.as_ref())?;
                    if let Ok(task) = BenchmarkTask::parse_bash_script(content_str) {
                        tasks.push(task);
                    }
                }
            }
        }
        tasks.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(tasks)
    }

    pub fn load_from_directory(dir: &Path) -> Result<Vec<BenchmarkTask>> {
        if !dir.exists() || !dir.is_dir() {
            return Err(anyhow!("Directory does not exist: {}", dir.display()));
        }

        let mut tasks = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "sh" {
                        let content = fs::read_to_string(&path)?;
                        if let Ok(task) = BenchmarkTask::parse_bash_script(&content) {
                            tasks.push(task);
                        }
                    }
                }
            }
        }

        tasks.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_load_from_directory() {
        let dir = tempdir().unwrap();
        let task_file = dir.path().join("task-test.sh");
        let content = r#"# id: task-test
# name: Test Task
# prompt: Hello test

# === SETUP ===
echo "setup"

# === VALIDATE ===
echo "validate"
"#;
        fs::write(&task_file, content).unwrap();

        let tasks = TaskLoader::load_from_directory(dir.path()).unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "task-test");
    }
}
