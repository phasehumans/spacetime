# Spacetime Context & Domain Glossary

## Core Concepts

### Spacetime Engine
The core execution harness responsible for orchestrating agent evaluations, Docker sandboxes, and benchmark evaluations.

### Execution Mode
- **Interactive TUI**: A terminal user interface rendered via `ratatui` when invoked without headless execution flags. Shows live agent reasoning, sandbox execution status, and visual benchmark metrics.
- **Headless CLI**: Command-line execution mode tailored for scripting, automated evaluation suites, and CI/CD pipelines.

### Benchmark Task
A self-contained system challenge evaluated inside an ephemeral Docker container.
- **Format**: Native Bash `.sh` files with header comments for metadata (`# id: ...`, `# prompt: ...`) and explicit section markers (`# === SETUP ===` and `# === VALIDATE ===`).
- **Embedded Suite**: Built-in suite of converted `.sh` benchmark tasks embedded into the compiled binary.
- **External Suite**: Custom `.sh` task files loaded from disk.

### Sandbox Runtime
An isolated containerized execution environment managed asynchronously via `bollard`.
- **Socket Auto-Detection**: Connects to Docker or Podman Unix sockets.
- **RAII Teardown Guard**: Ensures immediate container cleanup on completion, panic, timeout, or OS interrupt signal (Ctrl+C).
- **Stream Demuxing**: Distinguishes between stdout and stderr byte streams with exact exit status capture.

### Agent
An autonomous actor powered by an LLM Provider that receives environment state (stdout, stderr, exit codes) and outputs commands to execute within the sandbox.

### LLM Provider
An asynchronous provider abstraction (`LlmProvider` trait) supporting diverse AI services:
- **First-Class Providers**: OpenAI, Anthropic, Google Gemini, Ollama (Local), OpenRouter.
- **Generic OpenAI-Compatible**: Custom endpoints (DeepSeek, Groq, vLLM, LM Studio, Azure).

### Configuration Hierarchy
Provider configuration and API keys resolve in order of precedence:
1. CLI flags (`--provider`, `--model`, `--api-key`)
2. Environment variables (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GEMINI_API_KEY`, `OLLAMA_BASE_URL`)
3. Project config file (`spacetime.toml`)
4. Global user config file (`~/.config/spacetime/config.toml`)

### Multi-Channel Distribution
- **Cargo**: Published `spacetime-cli` package on crates.io.
- **Curl Installer**: Cross-platform shell script (`install.sh`) fetching prebuilt GitHub release binaries.
- **NPM Package**: `npx spacetime` / `npm i -g spacetime` native binary bridge.

### Quality Assurance & Testing Architecture
- **Unit Suite**: Task parser, config resolution, structured output extraction.
- **LLM API Mocks**: `wiremock` local HTTP mock servers testing provider clients deterministically.
- **Sandbox Integration**: Ephemeral Docker container initialization and RAII teardown tests.
- **CI Gates**: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
