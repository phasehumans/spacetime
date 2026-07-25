use clap::Parser;
use spacetime_cli::cli::{Cli, Commands};

#[test]
fn test_cli_parse_list() {
    let args = vec!["spacetime", "list"];
    let cli = Cli::parse_from(args);
    assert_eq!(cli.command, Some(Commands::List));
}

#[test]
fn test_cli_parse_eval() {
    let args = vec!["spacetime", "eval", "--task", "task-001", "--json"];
    let cli = Cli::parse_from(args);
    assert_eq!(
        cli.command,
        Some(Commands::Eval {
            task: Some("task-001".to_string()),
            json: true,
            full_screen: false,
        })
    );
}
