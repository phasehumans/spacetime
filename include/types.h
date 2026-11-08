#ifndef SPACETIME_TYPES_H
#define SPACETIME_TYPES_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define SPACETIME_VERSION "0.0.3"
#define DEFAULT_SANDBOX_IMAGE "spacetime-sandbox:latest"
#define DEFAULT_SUBMISSIONS_DIR "leaderboard/submissions"

typedef struct {
    char id[128];
    char name[256];
    char description[512];
    char base_image[128];
    char *prompt;
    char task_dir[512];
    char setup_script[512];
    char test_script[512];
    uint32_t max_turns;
    uint64_t timeout_secs;
} BenchmarkTask;

typedef struct {
    char *stdout_buf;
    size_t stdout_len;
    char *stderr_buf;
    size_t stderr_len;
    int exit_code;
    bool timed_out;
    uint64_t duration_ms;
} ExecutionResult;

typedef struct {
    char task_id[128];
    char task_name[256];
    bool passed;
    double duration_secs;
    char agent_name[256];
    int exit_code;
    bool has_exit_code;
    char *error_message;
    char *agent_output;
    size_t prompt_tokens;
    size_t completion_tokens;
    size_t total_tokens;
    double estimated_cost_usd;
} TaskResult;

typedef struct {
    double first_attempt_rate;
    size_t first_attempt_count;
    double error_recovery_rate;
    size_t recovered_count;
    size_t tasks_with_errors;
    double self_verification_rate;
    size_t verified_count;
    double context_hygiene_score;
    size_t total_prompt_tokens;
    size_t total_completion_tokens;
    size_t total_tokens;
    double total_cost_usd;
    double cost_per_resolved_task;
} IntelligenceProfile;

typedef struct {
    char timestamp[64];
    char agent[256];
    char harness[128];
    char model[128];
    char spacetime_version[32];
    char taskset_id[64];
    char taskset_hash[65];
    size_t taskset_size;
    char sandbox[128];
    size_t total_tasks;
    size_t passed_tasks;
    size_t failed_tasks;
    double pass_rate;
    double total_duration_secs;
    size_t total_tokens;
    double total_cost_usd;
    double cost_per_resolved_task;
    TaskResult *results;
    size_t results_count;
    IntelligenceProfile intelligence;
    bool has_intelligence;
} BenchmarkSuiteResult;

typedef struct {
    char id[64];
    char hash[65];
    size_t size;
} TasksetIdentity;

typedef enum {
    HARNESS_CLAUDE_CODE,
    HARNESS_GEMINI_CLI,
    HARNESS_ANTIGRAVITY,
    HARNESS_CODEX,
    HARNESS_AIDER,
    HARNESS_DEVIN,
    HARNESS_DECEMBER,
    HARNESS_PI,
    HARNESS_CURSOR_CLI,
    HARNESS_SWE_AGENT,
    HARNESS_OPEN_HANDS,
    HARNESS_GOOSE,
    HARNESS_PLANDEX,
    HARNESS_CLINE,
    HARNESS_SMOLAGENTS,
    HARNESS_MENTAT,
    HARNESS_CUSTOM,
} HarnessType;

typedef struct {
    HarnessType harness;
    char name[256];
    char model[128];
    char custom_cmd[512];
    char mount_dir[512];
    double prompt_cost_per_m;
    double completion_cost_per_m;
} AgentProfile;

void free_execution_result(ExecutionResult *res);
void free_task_result(TaskResult *res);
void free_benchmark_suite_result(BenchmarkSuiteResult *suite);

#endif /* SPACETIME_TYPES_H */
