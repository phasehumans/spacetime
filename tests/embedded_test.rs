use spacetime_cli::embedded::TaskLoader;
use std::fs;
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
