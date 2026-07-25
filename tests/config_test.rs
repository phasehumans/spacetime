use spacetime_cli::config::ConfigResolver;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_resolve_default_config() {
    let config = ConfigResolver::resolve(None, None, None, None).unwrap();
    assert_eq!(config.provider, "openai");
    assert_eq!(config.model, "gpt-4o");
}

#[test]
fn test_resolve_cli_overrides_toml() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"
provider = "gemini"
model = "gemini-1.5-pro"
max_turns = 10
"#
    )
    .unwrap();

    let config = ConfigResolver::resolve(
        Some("anthropic".to_string()),
        Some("claude-3-5-sonnet-20241022".to_string()),
        None,
        Some(file.path()),
    )
    .unwrap();

    assert_eq!(config.provider, "anthropic");
    assert_eq!(config.model, "claude-3-5-sonnet-20241022");
    assert_eq!(config.max_turns, 10);
}
