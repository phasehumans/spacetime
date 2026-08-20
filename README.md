# ✱ Spacetime

Spacetime is a benchmark for evaluating AI agents on interactive terminal tasks.
It executes agents inside isolated Docker containers to solve realistic Linux problems and tests if their solutions actually work.

- **Clean Docker Sandboxes:** Every task runs in a fresh, isolated container so nothing leaks between test runs.
- **Works with Any Agent:** Ready-to-use support for Claude Code, Gemini CLI, Codex, Aider, OpenHands, and custom agent scripts.
- **Smart Performance Insights:** Tracks pass rates, speed, error recovery, and how well agents verify their own work.
- **Simple Interactive CLI:** A step-by-step terminal wizard to pick your agent, choose models, and run benchmarks in seconds.

```bash
curl -fsSL https://spacetime.trydecember.com | bash
```

```mermaid
sequenceDiagram
    autonumber
    participant CLI as Spacetime CLI
    box Hermetic Docker Sandbox
        participant Agent as In-Container AI Agent
        participant Env as Linux Environment (/root)
    end
    participant Telemetry as Telemetry Engine

    CLI->>Env: Spin up container & execute setup.sh
    CLI->>Agent: Spawn in-container agent with objective prompt
    
    loop In-Container Execution (Max Turns / Timeout)
        Agent->>Env: Run bash commands & edit files
        Env-->>CLI: Stream real-time stdout / stderr
        Agent->>Env: Run self-verification checks (curl, diff, status)
    end
    Agent-->>CLI: Agent process exits (output & exit code)

    CLI->>Env: Execute test.sh (ground-truth validation)
    Env-->>CLI: Verdict (exit 0 = Pass / exit 1 = Fail)
    CLI->>Env: Destroy and purge container

    CLI->>Telemetry: Compute resolution rate, latency & intelligence metrics
    Telemetry-->>CLI: Render scorecard & save JSON report
```
