# Spacetime Rust Rewrite & Multi-Provider Architecture Specification

## Problem Statement

Users evaluating autonomous AI agents on shell, system, and CLI capabilities currently face significant limitations with the TypeScript implementation of Spacetime:
- **Limited LLM Provider Support**: Only Google Gemini and OpenAI are supported, excluding Anthropic Claude, local Ollama models, OpenRouter, and generic OpenAI-compatible custom endpoints (DeepSeek, Groq, vLLM).
- **Node.js Runtime Dependency**: Running Spacetime requires Node.js, `npm`, `tsx`, and complex package management instead of a single static binary.
- **Task Formatting & Authoring Complexity**: Benchmark tasks are written in verbose YAML files containing embedded multi-line string scripts, making shell syntax highlighting, linting (`shellcheck`), and local execution awkward.
- **Basic UI/UX**: Output is limited to simple terminal line logging without a real-time visual dashboard, interactive scorecards, or streaming container output.
- **Single Packaging Channel**: The tool cannot be easily installed via standard package managers (`cargo install`, `npx spacetime`, or `curl | sh`).

## Solution

Spacetime will be completely rewritten as a high-performance, single-binary Rust application featuring:
1. **Hybrid Interface**: A full-screen interactive `ratatui` Terminal User Interface (TUI) for rich real-time agent observation alongside a scriptable headless CLI mode for automated benchmark pipelines and CI/CD.
2. **Unified Multi-Provider Engine**: An asynchronous `LlmProvider` trait supporting OpenAI, Anthropic, Google Gemini, Ollama, OpenRouter, and generic OpenAI-compatible endpoints with hierarchical configuration resolution.
3. **Pure Bash Task Specification**: Standardized executable `.sh` task scripts with shell header metadata comments and section markers (`# === SETUP ===` and `# === VALIDATE ===`). Default task suite is embedded directly into the compiled binary.
4. **Robust Sandbox Runtime**: Async container management via `bollard` connecting to Docker/Podman sockets with an RAII guard guaranteeing immediate container teardown on completion, error, or OS interrupt signal.
5. **Multi-Channel Distribution**: Automated cross-platform binary releases downloadable via Cargo (`crates.io`), NPM binary bridge (`npx spacetime`), and a single-line shell installer script (`curl | sh`).
6. **Multi-Tiered Testing Suite**: Unit tests, `wiremock` HTTP mock servers for LLM API testing, and container sandbox integration tests.

## User Stories

1. As a benchmark engineer, I want a single compile-time binary with zero runtime dependencies, so that I can evaluate agents on any machine with Docker installed.
2. As an AI developer, I want to evaluate agents against Anthropic Claude 3.5 Sonnet, so that I can compare its system command execution against OpenAI and Gemini models.
3. As a local LLM enthusiast, I want to test agents using self-hosted Ollama or vLLM endpoints, so that I can benchmark open-source models without sending data to external APIs.
4. As a DevOps engineer, I want to run `spacetime eval --json` in headless mode within a CI pipeline, so that I can automatically test agent regressions on every pull request.
5. As a terminal user, I want an interactive `ratatui` dashboard displaying live agent thoughts, container stdout/stderr streams, and step counters, so that I can visually inspect agent behavior during evaluation runs.
6. As a task author, I want to write benchmark tasks as standard `.sh` Bash scripts with syntax highlighting and linting support, so that creating new benchmark scenarios is effortless.
7. As a task author, I want clear `# === SETUP ===` and `# === VALIDATE ===` section markers in `.sh` task files, so that sandbox initialization and test validation logic are cleanly separated.
8. As a user on macOS or Linux, I want to install Spacetime using `curl -fsSL ... | sh`, so that I can set up the tool in seconds without manual compilation.
9. As a Node.js developer, I want to execute `npx spacetime eval`, so that I can use Spacetime directly within my existing JavaScript/TypeScript repository workflows.
10. As a Rust developer, I want to run `cargo install spacetime-cli`, so that I can install and manage Spacetime via standard Cargo toolchains.
11. As a security-conscious engineer, I want containers to automatically shut down and self-destruct if the process is killed or interrupted via Ctrl+C, so that no orphaned containers are left consuming host resources.
12. As a system operator, I want Spacetime to automatically detect whether Docker or Podman Unix sockets are present, so that I do not need to manually configure socket paths.
13. As a developer, I want API key credentials to be resolved in order of CLI flags > environment variables > project config (`spacetime.toml`) > global config (`~/.config/spacetime/config.toml`), so that I can manage configuration flexibly.
14. As an enterprise developer, I want to target custom OpenAI-compatible endpoints (such as DeepSeek, Groq, LM Studio, or Azure), so that I can benchmark proprietary or internal LLM deployments.
15. As a developer contributing to Spacetime, I want an offline HTTP mock test suite powered by `wiremock`, so that I can run full test suites without incurring LLM API token costs.
16. As a benchmark maintainer, I want built-in tasks to be embedded into the binary at compile time, so that the executable works out-of-the-box without requiring external asset files.
17. As an agent evaluator, I want strict turn limits and execution timeouts enforced per task, so that malfunctioning agents do not run infinitely in a loop.
18. As an evaluator, I want detailed execution scorecards showing pass/fail status, total execution duration, and command counts, so that I can quantitatively compare model capabilities.

## Implementation Decisions

### Core Architecture & Modules
- **Execution Harness Module**: Controls evaluation orchestration, loop management, turn limits (default 15), and agent state accumulation.
- **TUI Module (`ratatui`)**: Renders full-screen terminal dashboards using a double-buffered alternate screen mode, featuring split view widgets for live agent thoughts, container output logs, and scorecard status.
- **CLI Module (`clap`)**: Parses command line arguments, flags, and subcommands (`eval`, `list`, `ui`, `config`).
- **Sandbox Engine Module (`bollard`)**: Manages container creation, command execution attachment, stream demuxing (stdout vs. stderr), and exit status collection over Unix sockets. Implements RAII `Drop` guards for guaranteed container destruction.
- **Provider Abstraction Module**: Exposes an `async-trait` `LlmProvider` interface with concrete implementations for OpenAI, Anthropic, Google Gemini, Ollama, OpenRouter, and Generic OpenAI-Compatible endpoints.
- **Task Parsing Module**: Parses single-file `.sh` Bash tasks with metadata header comment extraction (`# id:`, `# name:`, `# prompt:`, `# base_image:`, `# max_turns:`, `# timeout:`) and section marker splitting (`# === SETUP ===` and `# === VALIDATE ===`). Includes embedded asset loading via compile-time binary embedding.
- **Configuration Resolution Module**: Merges CLI arguments, environment variables, project-level `spacetime.toml`, and global `~/.config/spacetime/config.toml`.

### API & Interface Contracts
- **LLM Provider Trait**: Defines an asynchronous `send_message` interface accepting structured agent turn history and returning an agent decision object containing `reasoning` text and `command` payload.
- **Sandbox Engine Interface**: Exposes asynchronous `initialize()`, `execute_command(cmd: &str)`, and `destroy()` methods.
- **Task Loader Interface**: Exposes `load_embedded_tasks()` and `load_from_directory(path: &Path)`.

### Schemas
- **Task Schema**: Executable `.sh` file format containing metadata headers followed by `# === SETUP ===` and `# === VALIDATE ===` code blocks.
- **Config Schema (`spacetime.toml`)**: Key-value format configuring default provider, model names, timeout settings, and custom endpoint URLs.

## Testing Decisions

### Good Test Principles
- Tests verify external behavior, API contracts, and state outputs rather than internal struct fields or implementation private methods.
- LLM API integrations are tested deterministically without network calls using local mock HTTP servers.

### Modules Tested
- **Task Script Parser**: Verifies metadata extraction from shell comments and section block splitting across valid and malformed `.sh` task scripts.
- **Config Hierarchy Resolver**: Verifies precedence order across flags, environment variables, and TOML configuration files.
- **LLM Provider Implementations**: Verifies JSON payload construction, authentication header generation, and tool/command response parsing against `wiremock` mock HTTP endpoints.
- **Sandbox Container Engine**: Verifies container initialization, command execution return codes, stdout/stderr stream separation, and RAII cleanup upon struct drop.

### Prior Art
- Standard Rust testing conventions using `#[tokio::test]`, `wiremock::MockServer`, and `bollard` integration test patterns.

## Out of Scope

- Remote cloud execution of Docker containers (execution is local to the machine running Docker/Podman).
- GUI desktop application (Spacetime is strictly a terminal application).
- Multi-node distributed agent evaluation clusters (single-node evaluation engine).

## Further Notes

- Architecture decisions are recorded in [ADR 0001](file:///home/chaitanya/code/spacetime/docs/adr/0001-rust-architecture-and-native-bash-tasks.md).
- System domain concepts are detailed in [CONTEXT.md](file:///home/chaitanya/code/spacetime/CONTEXT.md).
