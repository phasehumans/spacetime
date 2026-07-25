use spacetime_cli::task::BenchmarkTask;

#[test]
fn test_parse_valid_bash_task() {
    let script = r#"#!/usr/bin/env spacetime
# id: task-001
# name: Fix Nginx Configuration
# description: The agent needs to resolve a syntax error in /etc/nginx/nginx.conf.
# base_image: ubuntu:22.04
# max_turns: 15
# timeout: 300
# prompt: Fix nginx configuration syntax error.

# === SETUP ===
apt-get update && apt-get install -y nginx

# === VALIDATE ===
nginx -t
"#;

    let task = BenchmarkTask::parse_bash_script(script).unwrap();
    assert_eq!(task.id, "task-001");
    assert_eq!(task.name, "Fix Nginx Configuration");
    assert_eq!(task.base_image, "ubuntu:22.04");
    assert_eq!(task.max_turns, 15);
    assert_eq!(task.timeout_seconds, 300);
    assert!(task.setup_script.contains("apt-get update"));
    assert_eq!(task.validation_script, "nginx -t");
}

#[test]
fn test_parse_task_missing_validate_fails() {
    let script = r#"
# id: task-invalid
# name: Invalid Task
"#;
    let res = BenchmarkTask::parse_bash_script(script);
    assert!(res.is_err());
}
