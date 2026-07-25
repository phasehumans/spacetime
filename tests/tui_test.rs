use spacetime_cli::tui::TuiDashboard;

#[test]
fn test_tui_state_initialization() {
    let dashboard = TuiDashboard::new(
        "task-001".to_string(),
        "openai".to_string(),
        "gpt-4o".to_string(),
        15,
    )
    .unwrap();

    assert_eq!(dashboard.state.task_id, "task-001");
    assert_eq!(dashboard.state.provider, "openai");
    assert_eq!(dashboard.state.max_turns, 15);
}
