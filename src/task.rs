use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result, anyhow};
use include_dir::{include_dir, Dir};

use crate::types::BenchmarkTask;

pub static EMBEDDED_TASKS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/tasks");

pub fn get_tasks_directory(requested_path: &Path) -> Result<PathBuf> {
    if requested_path.exists() && requested_path != Path::new("") {
        return Ok(requested_path.to_path_buf());
    }

    let default_cache = get_default_tasks_cache_dir();
    ensure_embedded_tasks_extracted(&default_cache)?;
    Ok(default_cache)
}

pub fn get_default_tasks_cache_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".spacetime").join("tasks")
    } else {
        std::env::temp_dir().join("spacetime_tasks")
    }
}

pub fn ensure_embedded_tasks_extracted(target_dir: &Path) -> Result<()> {
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)
            .with_context(|| format!("Failed to create tasks cache directory at {}", target_dir.display()))?;
    }

    EMBEDDED_TASKS.extract(target_dir)
        .with_context(|| format!("Failed to extract embedded tasks into {}", target_dir.display()))?;

    Ok(())
}

pub fn load_all_tasks(tasks_dir: &Path) -> Result<Vec<BenchmarkTask>> {
    let resolved_dir = get_tasks_directory(tasks_dir)?;

    let mut tasks = Vec::new();
    let entries = fs::read_dir(&resolved_dir)
        .with_context(|| format!("Failed to read tasks directory: {}", resolved_dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && (path.join("prompt.txt").exists() || path.join("meta.sh").exists() || path.join("meta.toml").exists()) {
            match load_task_from_dir(&path) {
                Ok(task) => tasks.push(task),
                Err(e) => {
                    eprintln!("Warning: Failed to load task from {}: {}", path.display(), e);
                }
            }
        }
    }

    tasks.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(tasks)
}

pub fn find_task_by_id(tasks_dir: &Path, id_query: &str) -> Result<BenchmarkTask> {
    let tasks = load_all_tasks(tasks_dir)?;
    
    if let Some(task) = tasks.iter().find(|t| t.id == id_query) {
        return Ok(task.clone());
    }

    let clean_query = id_query.trim_start_matches("task-");
    let matches: Vec<_> = tasks
        .iter()
        .filter(|t| {
            t.id.contains(id_query)
                || t.id.trim_start_matches("task-").starts_with(clean_query)
                || t.name.to_lowercase().contains(&id_query.to_lowercase())
        })
        .collect();

    if matches.is_empty() {
        return Err(anyhow!("Task matching '{}' not found", id_query));
    }

    if matches.len() > 1 {
        if let Some(exact_prefix) = matches.iter().find(|t| t.id.starts_with(clean_query)) {
            return Ok((*exact_prefix).clone());
        }
    }

    Ok(matches[0].clone())
}

pub fn load_task_from_dir(dir: &Path) -> Result<BenchmarkTask> {
    let folder_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Invalid directory name: {}", dir.display()))?;

    let prompt_path = dir.join("prompt.txt");
    let setup_path = dir.join("setup.sh");
    let test_path = dir.join("test.sh");
    let meta_toml_path = dir.join("meta.toml");
    let meta_sh_path = dir.join("meta.sh");

    let prompt = if prompt_path.exists() {
        fs::read_to_string(&prompt_path)
            .with_context(|| format!("Failed to read prompt from {}", prompt_path.display()))?
            .trim()
            .to_string()
    } else {
        String::new()
    };

    let mut task_id = folder_name.to_string();
    let mut task_name = folder_name.to_string();
    let mut base_image = "ubuntu:22.04".to_string();
    let mut max_turns = 15;
    let mut timeout_secs = 30;
    let mut description = String::new();

    let meta_content_opt = if meta_toml_path.exists() {
        Some(fs::read_to_string(&meta_toml_path)?)
    } else if meta_sh_path.exists() {
        Some(fs::read_to_string(&meta_sh_path)?)
    } else {
        None
    };

    if let Some(meta_content) = meta_content_opt {
        for line in meta_content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim().to_uppercase();
                let val = val.trim().trim_matches('"').trim_matches('\'');
                match key.as_str() {
                    "TASK_ID" | "ID" => task_id = val.to_string(),
                    "TASK_NAME" | "NAME" => task_name = val.to_string(),
                    "BASE_IMAGE" | "IMAGE" => base_image = val.to_string(),
                    "MAX_TURNS" => {
                        if let Ok(n) = val.parse::<u32>() {
                            max_turns = n;
                        }
                    }
                    "TIMEOUT_SECS" | "TIMEOUT" => {
                        if let Ok(n) = val.parse::<u64>() {
                            timeout_secs = n;
                        }
                    }
                    "DESCRIPTION" | "DESC" => description = val.to_string(),
                    _ => {}
                }
            }
        }
    }

    if !setup_path.exists() {
        return Err(anyhow!("Missing setup.sh in {}", dir.display()));
    }
    if !test_path.exists() {
        return Err(anyhow!("Missing test.sh in {}", dir.display()));
    }

    Ok(BenchmarkTask {
        id: task_id,
        name: task_name,
        description,
        base_image,
        prompt,
        task_dir: dir.to_path_buf(),
        setup_script: setup_path,
        test_script: test_path,
        max_turns,
        timeout_secs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_all_tasks() {
        let tasks_dir = Path::new("tasks");
        let tasks = load_all_tasks(tasks_dir).expect("Failed to load tasks");
        assert_eq!(tasks.len(), 20, "Expected 20 benchmark tasks");
    }

    #[test]
    fn test_load_embedded_tasks_fallback() {
        let nonexistent_dir = Path::new("/nonexistent_spacetime_tasks_dir_12345");
        let tasks = load_all_tasks(nonexistent_dir).expect("Failed to load embedded tasks on fallback");
        assert_eq!(tasks.len(), 20, "Expected 20 embedded tasks loaded from binary");
    }

    #[test]
    fn test_find_task_by_id_fuzzy() {
        let tasks_dir = Path::new("tasks");
        let t1 = find_task_by_id(tasks_dir, "001-nginx-config").unwrap();
        assert_eq!(t1.id, "001-nginx-config");

        let t2 = find_task_by_id(tasks_dir, "001").unwrap();
        assert_eq!(t2.id, "001-nginx-config");

        let t3 = find_task_by_id(tasks_dir, "nginx").unwrap();
        assert_eq!(t3.id, "001-nginx-config");
    }

    #[test]
    fn test_task_script_integrity() {
        let tasks_dir = Path::new("tasks");
        let tasks = load_all_tasks(tasks_dir).unwrap();
        for t in tasks {
            assert!(!t.prompt.is_empty(), "Task {} must have a prompt", t.id);
            assert!(t.setup_script.exists(), "Task {} missing setup.sh", t.id);
            assert!(t.test_script.exists(), "Task {} missing test.sh", t.id);
        }
    }

    #[test]
    fn test_load_task_from_meta_toml() {
        let temp_dir = std::env::temp_dir().join(format!("spacetime_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = fs::create_dir_all(&temp_dir);
        let _ = fs::write(temp_dir.join("prompt.txt"), "Test prompt");
        let _ = fs::write(temp_dir.join("setup.sh"), "#!/bin/bash\necho setup");
        let _ = fs::write(temp_dir.join("test.sh"), "#!/bin/bash\nexit 0");
        let _ = fs::write(
            temp_dir.join("meta.toml"),
            "TASK_ID = \"999-custom-task\"\nTASK_NAME = \"Custom Task\"\nMAX_TURNS = 20\nTIMEOUT_SECS = 45\nDESCRIPTION = \"A test task\"\n",
        );

        let task = load_task_from_dir(&temp_dir).expect("Should load task with meta.toml");
        assert_eq!(task.id, "999-custom-task");
        assert_eq!(task.name, "Custom Task");
        assert_eq!(task.max_turns, 20);
        assert_eq!(task.timeout_secs, 45);
        assert_eq!(task.description, "A test task");
        assert_eq!(task.prompt, "Test prompt");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
