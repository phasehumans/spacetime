<p align="center">
  <img src="website/logo.svg" alt="spacetime" height="30" />
</p>

<h4 align="center">a benchmark for evaluating ai agents on interactive terminal tasks</h4>

spacetime evaluates ai agents inside isolated docker containers on real terminal challenges like fixing broken nginx servers, resolving git conflicts, parsing logs, and repairing port clashes. solutions are tested against strict test assertions to verify what actually works.

```bash
curl -fsSL https://spacetime.trydecember.com | bash
```

### how it works

- starts a fresh, isolated docker container
- gives the agent terminal access to fix the task
- streams live terminal output in real time
- runs automated tests to verify the solution
- cleans up and generates the final scorecard

```mermaid
sequenceDiagram
    autonumber
    participant CLI as spacetime cli
    box hermetic docker sandbox
        participant Agent as in-container ai agent
        participant Env as linux environment (/root)
    end
    participant Telemetry as telemetry engine

    CLI->>Env: spin up container & execute setup.sh
    CLI->>Agent: spawn in-container agent with objective prompt
    
    loop in-container execution (max turns / timeout)
        Agent->>Env: run bash commands & edit files
        Env-->>CLI: stream real-time stdout / stderr
        Agent->>Env: run self-verification checks (curl, diff, status)
    end
    Agent-->>CLI: agent process exits (output & exit code)

    CLI->>Env: execute test.sh (ground-truth validation)
    Env-->>CLI: verdict (exit 0 = pass / exit 1 = fail)
    CLI->>Env: destroy and purge container

    CLI->>Telemetry: compute resolution rate, latency & intelligence metrics
    Telemetry-->>CLI: render scorecard & save json report
```
