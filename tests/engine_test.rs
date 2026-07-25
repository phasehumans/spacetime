use spacetime_cli::engine::EvaluationScorecard;

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
