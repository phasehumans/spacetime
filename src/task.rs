use std::fs;
use std::path::Path;
use anyhow::{Context, Result, anyhow};
use crate::types::BenchmarkTask;

pub fn load_all_tasks(tasks_dir: &Path) -> Result<Vec<BenchmarkTask>> {
    if !tasks_dir.exists() {
        return Err(anyhow!("Tasks directory '{}' does not exist", tasks_dir.display()));
    }

    let mut tasks = Vec::new();
    let entries = fs::read_dir(tasks_dir)
        .with_context(|| format!("Failed to read tasks directory: {}", tasks_dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.join("prompt.txt").exists() || path.join("meta.sh").exists() {
                match load_task_from_dir(&path) {
                    Ok(task) => tasks.push(task),
                    Err(e) => {
                        eprintln!("Warning: Failed to load task from {}: {}", path.display(), e);
                    }
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
    let meta_path = dir.join("meta.sh");

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

    if meta_path.exists() {
        let meta_content = fs::read_to_string(&meta_path)?;
        for line in meta_content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let val = val.trim().trim_matches('"').trim_matches('\'');
                match key {
                    "TASK_ID" => task_id = val.to_string(),
                    "TASK_NAME" => task_name = val.to_string(),
                    "BASE_IMAGE" => base_image = val.to_string(),
                    "MAX_TURNS" => {
                        if let Ok(n) = val.parse::<u32>() {
                            max_turns = n;
                        }
                    }
                    "TIMEOUT_SECS" => {
                        if let Ok(n) = val.parse::<u64>() {
                            timeout_secs = n;
                        }
                    }
                    "DESCRIPTION" => description = val.to_string(),
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
        if tasks_dir.exists() {
            let tasks = load_all_tasks(tasks_dir).expect("Failed to load tasks");
            assert_eq!(tasks.len(), 20, "Expected 20 benchmark tasks");
        }
    }

    #[test]
    fn test_find_task_by_id_fuzzy() {
        let tasks_dir = Path::new("tasks");
        if tasks_dir.exists() {
            let t1 = find_task_by_id(tasks_dir, "001-nginx-config").unwrap();
            assert_eq!(t1.id, "001-nginx-config");

            let t2 = find_task_by_id(tasks_dir, "001").unwrap();
            assert_eq!(t2.id, "001-nginx-config");

            let t3 = find_task_by_id(tasks_dir, "nginx").unwrap();
            assert_eq!(t3.id, "001-nginx-config");
        }
    }

    #[test]
    fn test_task_script_integrity() {
        let tasks_dir = Path::new("tasks");
        if tasks_dir.exists() {
            let tasks = load_all_tasks(tasks_dir).unwrap();
            for t in tasks {
                assert!(!t.prompt.is_empty(), "Task {} must have a prompt", t.id);
                assert!(t.setup_script.exists(), "Task {} missing setup.sh", t.id);
                assert!(t.test_script.exists(), "Task {} missing test.sh", t.id);
            }
        }
    }
}
