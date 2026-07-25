use spacetime_cli::sandbox::SandboxRuntime;

#[tokio::test]
async fn test_sandbox_unit_creation() {
    let runtime = SandboxRuntime::new();
    assert!(runtime.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_sandbox_lifecycle_live_docker() {
    let runtime = SandboxRuntime::new().unwrap();
    let mut sandbox = runtime.create_sandbox("alpine:latest").await.unwrap();

    let result = sandbox.execute("echo 'hello spacetime'").await.unwrap();
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("hello spacetime"));

    sandbox.destroy().await.unwrap();
    assert!(sandbox.destroyed);
}
