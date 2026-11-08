#define _GNU_SOURCE
#include "eval.h"
#include "agent.h"
#include "docker.h"
#include "taskset.h"
#include "report.h"

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <time.h>
#include <unistd.h>

static double get_time_sec(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec + (double)ts.tv_nsec / 1e9;
}

static char *read_file_content(const char *path) {
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;
    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);
    if (sz < 0) {
        fclose(f);
        return NULL;
    }
    char *buf = malloc((size_t)sz + 1);
    if (!buf) {
        fclose(f);
        return NULL;
    }
    size_t n = fread(buf, 1, (size_t)sz, f);
    buf[n] = '\0';
    fclose(f);
    return buf;
}

static void save_task_log(const char *task_id, const char *agent_name, const char *output, bool passed) {
    mkdir("logs", 0755);
    time_t now = time(NULL);
    struct tm *tm = gmtime(&now);
    char filename[512];
    snprintf(filename, sizeof(filename), "logs/%s_%04d%02d%02d_%02d%02d%02d.log",
        task_id, tm->tm_year + 1900, tm->tm_mon + 1, tm->tm_mday,
        tm->tm_hour, tm->tm_min, tm->tm_sec);

    FILE *f = fopen(filename, "w");
    if (f) {
        fprintf(f, "========================================================\n");
        fprintf(f, "SPACETIME TASK EXECUTION LOG\n");
        fprintf(f, "Task:   %s\n", task_id);
        fprintf(f, "Agent:  %s\n", agent_name);
        fprintf(f, "Result: %s\n", passed ? "PASSED" : "FAILED");
        fprintf(f, "========================================================\n\n");
        fprintf(f, "%s\n", output ? output : "");
        fclose(f);
    }
}

int eval_run_task(const BenchmarkTask *task, const AgentProfile *agent, const char *image, uint64_t timeout_override, bool silent, TaskResult *out_res) {
    memset(out_res, 0, sizeof(TaskResult));
    snprintf(out_res->task_id, sizeof(out_res->task_id), "%s", task->id);
    snprintf(out_res->task_name, sizeof(out_res->task_name), "%s", task->name);
    snprintf(out_res->agent_name, sizeof(out_res->agent_name), "%s", agent->name);

    double start_time = get_time_sec();

    if (!silent) {
        printf("\n\033[1;37m[Task]     %s\033[0m\n", task->name);
        printf("\033[36m[Agent]    %s\033[0m\n", agent->name);
        printf("\033[2m• Target Objective:\033[0m\n");
        printf("  \033[37m%s\033[0m\n", task->prompt ? task->prompt : "");
        printf("\033[2m• Initializing Docker sandbox (%s)...\033[0m\n", image);
    }

    char container_id[65];
    if (docker_create_sandbox(image, agent->mount_dir[0] ? agent->mount_dir : NULL, container_id) != 0) {
        out_res->error_message = strdup("Failed to create Docker sandbox container");
        out_res->duration_secs = get_time_sec() - start_time;
        return -1;
    }

    if (docker_start_container(container_id) != 0) {
        docker_remove_container(container_id, true);
        out_res->error_message = strdup("Failed to start Docker sandbox container");
        out_res->duration_secs = get_time_sec() - start_time;
        return -1;
    }

    // Run setup.sh
    if (!silent) printf("\033[2m• Executing setup.sh inside sandbox...\033[0m\n");
    char *setup_script = read_file_content(task->setup_script);
    if (!setup_script) {
        docker_remove_container(container_id, true);
        out_res->error_message = strdup("Missing or unreadable setup.sh");
        out_res->duration_secs = get_time_sec() - start_time;
        return -1;
    }

    ExecutionResult setup_res;
    int err = docker_execute_script(container_id, setup_script, 180, &setup_res);
    free(setup_script);

    if (err != 0 || setup_res.exit_code != 0) {
        char err_msg[1024];
        snprintf(err_msg, sizeof(err_msg), "setup.sh failed with exit code %d: %s",
            setup_res.exit_code, setup_res.stderr_buf ? setup_res.stderr_buf : "");
        out_res->error_message = strdup(err_msg);
        free_execution_result(&setup_res);
        docker_remove_container(container_id, true);
        out_res->duration_secs = get_time_sec() - start_time;
        return 0;
    }
    free_execution_result(&setup_res);

    if (!silent) printf("\033[32mEnvironment initialized (%s)\033[0m\n", task->id);

    // Run Agent
    char agent_cmd[2048];
    agent_profile_build_cmd(agent, task->prompt, agent_cmd, sizeof(agent_cmd));

    char **env_vars = NULL;
    size_t env_count = 0;
    agent_profile_get_env(agent, task->prompt, &env_vars, &env_count);

    uint64_t effective_timeout = timeout_override > 0 ? timeout_override : task->timeout_secs;
    if (effective_timeout == 0) effective_timeout = 300;

    if (!silent) {
        printf("\n\033[1;37m• Spawning In-Container Agent:\033[0m\n");
        printf("  \033[33m%s\033[0m\n\n", agent_cmd);
        printf("\033[2m--------------------------------------------------------\033[0m\n");
    }

    ExecutionResult agent_res;
    docker_execute_agent(container_id, agent_cmd, (const char **)env_vars, env_count, effective_timeout, silent, &agent_res);

    agent_profile_free_env(env_vars, env_count);

    if (!silent) {
        printf("\n\033[2m--------------------------------------------------------\033[0m\n");
    }

    char *scrubbed = docker_scrub_secrets(agent_res.stdout_buf ? agent_res.stdout_buf : "");
    out_res->agent_output = scrubbed;
    out_res->exit_code = agent_res.exit_code;
    out_res->has_exit_code = true;
    free_execution_result(&agent_res);

    // Run test.sh
    if (!silent) printf("\033[2m• Evaluating final container state with test.sh...\033[0m\n");
    char *test_script = read_file_content(task->test_script);
    if (!test_script) {
        docker_remove_container(container_id, true);
        out_res->error_message = strdup("Missing or unreadable test.sh");
        out_res->duration_secs = get_time_sec() - start_time;
        return -1;
    }

    ExecutionResult test_res;
    docker_execute_script(container_id, test_script, 60, &test_res);
    free(test_script);

    out_res->passed = (test_res.exit_code == 0);
    if (!out_res->passed) {
        out_res->error_message = strdup(test_res.stderr_buf && *test_res.stderr_buf ? test_res.stderr_buf : "test assertion failed");
    }
    free_execution_result(&test_res);

    docker_remove_container(container_id, true);

    double total_duration = get_time_sec() - start_time;
    out_res->duration_secs = total_duration;

    save_task_log(task->id, agent->name, out_res->agent_output, out_res->passed);

    size_t prompt_chars = (task->prompt ? strlen(task->prompt) : 0) + 1500;
    out_res->prompt_tokens = (size_t)ceil((double)prompt_chars / 3.8);
    size_t out_chars = out_res->agent_output ? strlen(out_res->agent_output) : 0;
    out_res->completion_tokens = out_chars == 0 ? 50 : (size_t)ceil((double)out_chars / 3.8);
    out_res->total_tokens = out_res->prompt_tokens + out_res->completion_tokens;
    out_res->estimated_cost_usd = ((double)out_res->prompt_tokens / 1e6) * agent->prompt_cost_per_m +
                                  ((double)out_res->completion_tokens / 1e6) * agent->completion_cost_per_m;

    if (!silent) {
        printf("\n\033[1m• SPACETIME EVALUATION •\033[0m\n");
        printf("  \033[2mTask:       \033[0m%s\n", task->name);
        printf("  \033[2mAgent:      \033[0m%s\n", agent->name);
        printf("  \033[2mDuration:   \033[0m%.2fs\n", total_duration);
        printf("  \033[2mTokens:     \033[0m%zu tokens ($%.4f USD)\n", out_res->total_tokens, out_res->estimated_cost_usd);
        if (out_res->passed) {
            printf("  \033[2mResult:     \033[1;32mPASSED\033[0m\n");
        } else {
            printf("  \033[2mResult:     \033[1;31mFAILED\033[0m\n");
        }
        printf("\033[2m========================================================\033[0m\n\n");
    }

    return 0;
}

int eval_run_suite(const BenchmarkTask *tasks, size_t count, const AgentProfile *agent, const char *image, uint64_t timeout_override, const char *output_path, BenchmarkSuiteResult *out_suite) {
    memset(out_suite, 0, sizeof(BenchmarkSuiteResult));
    double start_time = get_time_sec();

    time_t now = time(NULL);
    struct tm *tm = gmtime(&now);
    strftime(out_suite->timestamp, sizeof(out_suite->timestamp), "%Y-%m-%dT%H:%M:%SZ", tm);

    snprintf(out_suite->agent, sizeof(out_suite->agent), "%s", agent->name);
    snprintf(out_suite->harness, sizeof(out_suite->harness), "%s", agent->name);
    snprintf(out_suite->model, sizeof(out_suite->model), "%s", agent->model);
    snprintf(out_suite->spacetime_version, sizeof(out_suite->spacetime_version), "%s", SPACETIME_VERSION);
    snprintf(out_suite->sandbox, sizeof(out_suite->sandbox), "%s", image);

    TasksetIdentity id;
    if (taskset_compute(tasks, count, &id) == 0) {
        snprintf(out_suite->taskset_id, sizeof(out_suite->taskset_id), "%s", id.id);
        snprintf(out_suite->taskset_hash, sizeof(out_suite->taskset_hash), "%s", id.hash);
        out_suite->taskset_size = id.size;
    }

    out_suite->total_tasks = count;
    out_suite->results = malloc(sizeof(TaskResult) * count);
    out_suite->results_count = count;

    printf("\n\033[33m✱\033[0m  \033[1mSPACETIME BENCHMARK SUITE (%zu Tasks)\033[0m\n", count);
    printf("\033[2m│\033[0m  \033[1magent:\033[0m       \033[2m%s\033[0m\n", agent->name);
    printf("\033[2m│\033[0m  \033[1msandbox:\033[0m     \033[2m%s\033[0m\n", image);
    printf("\033[2m│\033[0m\n");

    for (size_t i = 0; i < count; i++) {
        const BenchmarkTask *t = &tasks[i];
        const char *prefix = (i == count - 1) ? "└─" : "├─";

        eval_run_task(t, agent, image, timeout_override, true, &out_suite->results[i]);
        TaskResult *r = &out_suite->results[i];

        if (r->passed) {
            out_suite->passed_tasks++;
            printf("\033[2m│  %s\033[0m \033[32mpass\033[0m \033[1m%-24s\033[0m \033[2m(%.1fs)\033[0m\n", prefix, t->id, r->duration_secs);
        } else {
            out_suite->failed_tasks++;
            char err_snippet[48] = "test assertion failed";
            if (r->error_message && *r->error_message) {
                char *first_nl = strchr(r->error_message, '\n');
                if (first_nl) *first_nl = '\0';
                strncpy(err_snippet, r->error_message, sizeof(err_snippet) - 1);
                err_snippet[sizeof(err_snippet) - 1] = '\0';
            }
            printf("\033[2m│  %s\033[0m \033[31mfail\033[0m \033[1m%-24s\033[0m \033[2m(%.1fs)\033[0m - \033[31m%s\033[0m\n", prefix, t->id, r->duration_secs, err_snippet);
        }

        out_suite->total_tokens += r->total_tokens;
        out_suite->total_cost_usd += r->estimated_cost_usd;
    }

    out_suite->total_duration_secs = get_time_sec() - start_time;
    out_suite->pass_rate = count > 0 ? ((double)out_suite->passed_tasks / (double)count) * 100.0 : 0.0;
    out_suite->cost_per_resolved_task = out_suite->passed_tasks > 0 ? out_suite->total_cost_usd / (double)out_suite->passed_tasks : 0.0;

    printf("\n\033[1m• BENCHMARK SUMMARY •\033[0m\n");
    printf("  Passed:       %zu / %zu (%.1f%%)\n", out_suite->passed_tasks, count, out_suite->pass_rate);
    printf("  Duration:     %.2fs\n", out_suite->total_duration_secs);
    printf("  Total Cost:   $%.4f USD\n", out_suite->total_cost_usd);
    printf("  Cost/Resolve: $%.4f USD\n\n", out_suite->cost_per_resolved_task);

    char default_report_path[512];
    if (!output_path || !*output_path) {
        mkdir("results", 0755);
        snprintf(default_report_path, sizeof(default_report_path), "results/spacetime_results_%ld.json", (long)now);
        output_path = default_report_path;
    }

    report_save_json(out_suite, output_path);
    printf("\033[32mResults saved to %s\033[0m\n", output_path);

    return 0;
}
