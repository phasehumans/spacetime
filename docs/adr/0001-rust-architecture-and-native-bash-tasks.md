# 1. Rust Architecture, Native Bash Tasks, and Multi-Provider LLM Abstraction

- **Status**: Accepted
- **Date**: 2026-07-25

## Context

The original Spacetime project was written in TypeScript running on Node.js using `dockerode`, `inquirer`, and single-provider API calls. To turn Spacetime into a high-performance, single-file binary CLI and Ratatui TUI application, distribute it across NPM, Cargo, and `curl` installers, and support diverse LLM providers with robust offline/mock testing, a full architecture redesign was evaluated.

## Decision

We decided to rewrite Spacetime in Rust with the following core architectural choices:

1. **Hybrid Execution Engine**: A unified Rust binary supporting both an interactive full-screen `ratatui` TUI dashboard and a scriptable headless CLI mode.
2. **Async Docker Sandbox Engine (`bollard`)**: Control Docker/Podman sockets directly via `bollard` with an RAII guard to guarantee zero leaked containers on panic or `SIGINT`.
3. **Pure Bash Task Specification (`.sh`)**: Replace YAML task definitions with single-file executable `.sh` Bash scripts featuring shell header comments for metadata and `# === SETUP ===` / `# === VALIDATE ===` section markers. All default tasks are embedded into the binary at compile time.
4. **Unified `LlmProvider` Async Trait**: Support OpenAI, Anthropic, Gemini, Ollama, OpenRouter, and generic OpenAI-compatible custom endpoints with a hierarchical configuration system.
5. **Multi-Channel Distribution**: Automated release pipelines for `cargo install`, `npx spacetime` (via native binary bridge), and `curl | sh` shell installer script.
6. **Multi-Tiered Testing**: Combine unit tests, `wiremock` HTTP mock servers for LLM APIs, and container sandbox integration tests.

## Consequences

- **Positive**: Single binary delivery, zero Node.js runtime dependency required on end-user machine, ultra-fast container execution, native TUI experience, full provider support, testable offline.
- **Negative**: Requires Rust toolchain for development and cross-compilation pipeline setup for releases.
