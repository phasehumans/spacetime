#pragma once

#include <cstddef>
#include <cstdint>
#include <optional>
#include <string>
#include <vector>

namespace spacetime {

inline constexpr char kVersion[] = "0.0.3";
inline constexpr char kDefaultSandboxImage[] = "spacetime-sandbox:latest";

struct BenchmarkTask {
    std::string id;
    std::string name;
    std::string description;
    std::string base_image = "ubuntu:22.04";
    std::string prompt;
    std::string task_dir;
    std::string setup_script;
    std::string test_script;
    std::uint32_t max_turns = 15;
    std::uint64_t timeout_secs = 30;
};

struct ExecutionResult {
    std::string stdout_data;
    std::string stderr_data;
    int exit_code = 0;
    bool timed_out = false;
    std::uint64_t duration_ms = 0;
};

struct TaskResult {
    std::string task_id;
    std::string task_name;
    bool passed = false;
    double duration_secs = 0.0;
    std::string agent_name;
    std::optional<int> exit_code;
    std::optional<std::string> error_message;
    std::string agent_output;
    std::size_t prompt_tokens = 0;
    std::size_t completion_tokens = 0;
    std::size_t total_tokens = 0;
    double estimated_cost_usd = 0.0;
};

struct IntelligenceProfile {
    double first_attempt_rate = 0.0;
    std::size_t first_attempt_count = 0;
    double error_recovery_rate = 0.0;
    std::size_t recovered_count = 0;
    std::size_t tasks_with_errors = 0;
    double self_verification_rate = 0.0;
    std::size_t verified_count = 0;
    double context_hygiene_score = 0.0;
    std::size_t total_prompt_tokens = 0;
    std::size_t total_completion_tokens = 0;
    std::size_t total_tokens = 0;
    double total_cost_usd = 0.0;
    double cost_per_resolved_task = 0.0;
};

struct BenchmarkSuiteResult {
    std::string timestamp;
    std::string agent;
    std::string harness;
    std::string model;
    std::string spacetime_version;
    std::string taskset_id;
    std::string taskset_hash;
    std::size_t taskset_size = 0;
    std::string sandbox;
    std::size_t total_tasks = 0;
    std::size_t passed_tasks = 0;
    std::size_t failed_tasks = 0;
    double pass_rate = 0.0;
    double total_duration_secs = 0.0;
    std::size_t total_tokens = 0;
    double total_cost_usd = 0.0;
    double cost_per_resolved_task = 0.0;
    std::vector<TaskResult> results;
    std::optional<IntelligenceProfile> intelligence;
};

struct TasksetIdentity {
    std::string id;
    std::string hash;
    std::size_t size = 0;
};

enum class HarnessType {
    ClaudeCode,
    GeminiCli,
    Antigravity,
    Codex,
    Aider,
    Devin,
    December,
    Pi,
    CursorCli,
    SweAgent,
    OpenHands,
    Goose,
    Plandex,
    Cline,
    Smolagents,
    Mentat,
    Custom,
};

struct AgentProfile {
    HarnessType harness = HarnessType::Custom;
    std::string name;
    std::string model;
    std::string custom_cmd;
    std::string mount_dir;
    double prompt_cost_per_m = 3.00;
    double completion_cost_per_m = 15.00;
};

} // namespace spacetime
