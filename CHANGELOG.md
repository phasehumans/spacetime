# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub Actions CI pipeline ([`.github/workflows/ci.yml`](.github/workflows/ci.yml)) testing cargo check, unit/integration tests, clippy, formatting, and sandbox Docker build.
- Crate metadata in [`Cargo.toml`](Cargo.toml) (`license`, `repository`, `readme`, `keywords`, `categories`).
- Root [`logo.svg`](logo.svg) asset.

### Changed
- CLI version is now dynamically synchronized with `Cargo.toml` using `env!("CARGO_PKG_VERSION")`.
- Improved Windows detection in [`install.sh`](install.sh) with clear instructions to run inside WSL 2.

### Fixed
- Redundant clippy pricing condition in [`src/agent/profile.rs`](src/agent/profile.rs).

## [0.0.3] - 2026-09-13

### Added
- Complete benchmark suite expanded to 50 embedded Linux environments covering sysadmin, networking, debugging, databases, and data pipelines.
- Deep telemetry tracking token economics, model cost estimation, turn efficiency, and error recovery.
- New `spacetime create-task` CLI command to scaffold benchmark challenges with metadata, setup, and test assertions.

### Fixed
- TUI display glitch and terminal cursor state management on exit.
- Hardened sandbox script ([`sandbox.sh`](sandbox.sh)) for process cleanup and container lifecycle management.

## [0.0.2] - 2026-08-20

### Added
- Non-root agent user execution (UID 1000) inside containers with passwordless sudo.
- Sensitive API key and authorization token scrubbing across logs and telemetry reports.
- Global SIGINT (`Ctrl+C`) signal handler and container process termination on task timeout.
- Safe prompt injection via `SPACETIME_PROMPT` and read-write workspace mounts.

### Fixed
- Multi-byte Unicode string truncation safety and NaN float `total_cmp` comparisons.
- Benchmark provisioning fixes for tasks 001, 004, 008, 011, 015, and 020.
- Dependency pruning and cleanup.

## [0.0.1] - 2026-08-20

### Added
- Initial release of Spacetime benchmark arena.
- Hermetic Docker container isolation for terminal coding agents.
- Interactive terminal wizard (`spacetime tui`) and CLI task runner.
- Binary embedded benchmark task suite via `include_dir`.
- One-liner curl installer script ([`install.sh`](install.sh)) with pre-compiled GitHub Release binary distribution.
