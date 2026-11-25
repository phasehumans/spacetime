#include "eval.hpp"

#include <algorithm>
#include <chrono>
#include <cmath>
#include <ctime>
#include <filesystem>
#include <format>
#include <fstream>
#include <optional>
#include <system_error>
#include <utility>

#include "agent.hpp"
#include "docker.hpp"
#include "report.hpp"
#include "taskset.hpp"
#include "util.hpp"

namespace spacetime {

namespace {

using Clock = std::chrono::steady_clock;

double seconds_since(Clock::time_point start) {
    return std::chrono::duration<double>(Clock::now() - start).count();
}

std::string format_utc(std::time_t when, const char* fmt) {
    std::tm tm{};
    gmtime_r(&when, &tm);
    char buf[64];
    const std::size_t n = std::strftime(buf, sizeof(buf), fmt, &tm);
    return std::string(buf, n);
}

void make_dir(const char* path) {
    std::error_code ec;
    std::filesystem::create_directory(path, ec);
}

// Removes the sandbox container when it goes out of scope.
class SandboxGuard {
public:
    explicit SandboxGuard(std::string container_id) : id_(std::move(container_id)) {}
    ~SandboxGuard() { docker::remove_container(id_, true); }

    SandboxGuard(const SandboxGuard&) = delete;
    SandboxGuard& operator=(const SandboxGuard&) = delete;

    const std::string& id() const { return id_; }

private:
    std::string id_;
};

void save_task_log(const std::string& task_id, const std::string& agent_name, const std::string& output,
                   bool passed) {
    make_dir("logs");
    const std::string filename =
        std::format("logs/{}_{}.log", task_id, format_utc(std::time(nullptr), "%Y%m%d_%H%M%S"));

    std::ofstream log(filename);
    if (!log) return;
    log << "========================================================\n"
        << "SPACETIME TASK EXECUTION LOG\n"
        << "Task:   " << task_id << "\n"
        << "Agent:  " << agent_name << "\n"
        << "Result: " << (passed ? "PASSED" : "FAILED") << "\n"
        << "========================================================\n\n"
        << output << "\n";
}

// Rough token estimate (~3.8 characters per token) until real usage numbers
// are captured from the harness.
std::size_t estimate_tokens(std::size_t chars) {
    return static_cast<std::size_t>(std::ceil(static_cast<double>(chars) / 3.8));
}

} // namespace

TaskResult eval_run_task(const BenchmarkTask& task, const AgentProfile& agent, const std::string& image,
                         std::uint64_t timeout_override, bool silent) {
    TaskResult res;
    res.task_id = task.id;
    res.task_name = task.name;
    res.agent_name = agent.name;

    const auto start_time = Clock::now();
    std::optional<SandboxGuard> sandbox;

    // Tears the sandbox down and finishes the result with an error.
    const auto fail = [&](std::string message) {
        sandbox.reset();
        res.error_message = std::move(message);
        res.duration_secs = seconds_since(start_time);
        return res;
    };

    if (!silent) {
        print("\n\033[1;37m[Task]     {}\033[0m\n", task.name);
        print("\033[36m[Agent]    {}\033[0m\n", agent.name);
        print("\033[2m• Target Objective:\033[0m\n");
        print("  \033[37m{}\033[0m\n", task.prompt);
        print("\033[2m• Initializing Docker sandbox ({})...\033[0m\n", image);
    }

    const auto container_id = docker::create_sandbox(image, agent.mount_dir);
    if (!container_id) return fail("Failed to create Docker sandbox container");
    sandbox.emplace(*container_id);

    if (!docker::start_container(sandbox->id())) {
        return fail("Failed to start Docker sandbox container");
    }

    // Run setup.sh
    if (!silent) print("\033[2m• Executing setup.sh inside sandbox...\033[0m\n");
    const auto setup_script = read_file(task.setup_script);
    if (!setup_script) return fail("Missing or unreadable setup.sh");

    const auto setup_res = docker::execute_script(sandbox->id(), *setup_script, 180);
    if (!setup_res || setup_res->exit_code != 0) {
        return fail(std::format("setup.sh failed with exit code {}: {}", setup_res ? setup_res->exit_code : -1,
                                setup_res ? setup_res->stderr_data : std::string()));
    }

    if (!silent) print("\033[32mEnvironment initialized ({})\033[0m\n", task.id);

    // Run Agent
    const std::string agent_cmd = agent_profile_build_cmd(agent);
    const std::vector<std::string> env_vars = agent_profile_get_env(task.prompt);

    std::uint64_t effective_timeout = timeout_override > 0 ? timeout_override : task.timeout_secs;
    if (effective_timeout == 0) effective_timeout = 300;

    if (!silent) {
        print("\n\033[1;37m• Spawning In-Container Agent:\033[0m\n");
        print("  \033[33m{}\033[0m\n\n", agent_cmd);
        print("\033[2m--------------------------------------------------------\033[0m\n");
        std::fflush(stdout);
    }

    const auto agent_res = docker::execute_agent(sandbox->id(), agent_cmd, env_vars, effective_timeout, silent);

    if (!silent) {
        print("\n\033[2m--------------------------------------------------------\033[0m\n");
    }

    if (agent_res) {
        res.agent_output = docker::scrub_secrets(agent_res->stdout_data);
        res.exit_code = agent_res->exit_code;
    }

    // Run test.sh
    if (!silent) print("\033[2m• Evaluating final container state with test.sh...\033[0m\n");
    const auto test_script = read_file(task.test_script);
    if (!test_script) return fail("Missing or unreadable test.sh");

    const auto test_res = docker::execute_script(sandbox->id(), *test_script, 60);
    res.passed = test_res && test_res->exit_code == 0;
    if (!res.passed) {
        res.error_message =
            (test_res && !test_res->stderr_data.empty()) ? test_res->stderr_data : "test assertion failed";
    }

    sandbox.reset();
    res.duration_secs = seconds_since(start_time);

    save_task_log(task.id, agent.name, res.agent_output, res.passed);

    res.prompt_tokens = estimate_tokens(task.prompt.size() + 1500);
    res.completion_tokens = res.agent_output.empty() ? 50 : estimate_tokens(res.agent_output.size());
    res.total_tokens = res.prompt_tokens + res.completion_tokens;
    res.estimated_cost_usd = (static_cast<double>(res.prompt_tokens) / 1e6) * agent.prompt_cost_per_m +
                             (static_cast<double>(res.completion_tokens) / 1e6) * agent.completion_cost_per_m;

    if (!silent) {
        print("\n\033[1m• SPACETIME EVALUATION •\033[0m\n");
        print("  \033[2mTask:       \033[0m{}\n", task.name);
        print("  \033[2mAgent:      \033[0m{}\n", agent.name);
        print("  \033[2mDuration:   \033[0m{:.2f}s\n", res.duration_secs);
        print("  \033[2mTokens:     \033[0m{} tokens (${:.4f} USD)\n", res.total_tokens, res.estimated_cost_usd);
        if (res.passed) {
            print("  \033[2mResult:     \033[1;32mPASSED\033[0m\n");
        } else {
            print("  \033[2mResult:     \033[1;31mFAILED\033[0m\n");
        }
        print("\033[2m========================================================\033[0m\n\n");
    }

    return res;
}

BenchmarkSuiteResult eval_run_suite(const std::vector<BenchmarkTask>& tasks, const AgentProfile& agent,
                                    const std::string& image, std::uint64_t timeout_override,
                                    const std::string& output_path) {
    BenchmarkSuiteResult suite;
    const auto start_time = Clock::now();
    const std::time_t now = std::time(nullptr);
    const std::size_t count = tasks.size();

    suite.timestamp = format_utc(now, "%Y-%m-%dT%H:%M:%SZ");
    suite.agent = agent.name;
    suite.harness = agent.name;
    suite.model = agent.model;
    suite.spacetime_version = kVersion;
    suite.sandbox = image;

    const TasksetIdentity identity = taskset_compute(tasks);
    suite.taskset_id = identity.id;
    suite.taskset_hash = identity.hash;
    suite.taskset_size = identity.size;

    suite.total_tasks = count;
    suite.results.reserve(count);

    print("\n\033[33m✱\033[0m  \033[1mSPACETIME BENCHMARK SUITE ({} Tasks)\033[0m\n", count);
    print("\033[2m│\033[0m  \033[1magent:\033[0m       \033[2m{}\033[0m\n", agent.name);
    print("\033[2m│\033[0m  \033[1msandbox:\033[0m     \033[2m{}\033[0m\n", image);
    print("\033[2m│\033[0m\n");
    std::fflush(stdout);

    for (std::size_t i = 0; i < count; ++i) {
        const BenchmarkTask& task = tasks[i];
        const char* prefix = (i == count - 1) ? "└─" : "├─";

        TaskResult& r = suite.results.emplace_back(eval_run_task(task, agent, image, timeout_override, true));

        if (r.passed) {
            suite.passed_tasks++;
            print("\033[2m│  {}\033[0m \033[32mpass\033[0m \033[1m{:<24}\033[0m \033[2m({:.1f}s)\033[0m\n", prefix,
                  task.id, r.duration_secs);
        } else {
            suite.failed_tasks++;
            std::string err_snippet = "test assertion failed";
            if (r.error_message && !r.error_message->empty()) {
                // The suite report keeps only the first line of a failure.
                r.error_message->resize(std::min(r.error_message->find('\n'), r.error_message->size()));
                err_snippet = r.error_message->substr(0, 47);
            }
            print("\033[2m│  {}\033[0m \033[31mfail\033[0m \033[1m{:<24}\033[0m \033[2m({:.1f}s)\033[0m - "
                  "\033[31m{}\033[0m\n",
                  prefix, task.id, r.duration_secs, err_snippet);
        }
        std::fflush(stdout);

        suite.total_tokens += r.total_tokens;
        suite.total_cost_usd += r.estimated_cost_usd;
    }

    suite.total_duration_secs = seconds_since(start_time);
    suite.pass_rate =
        count > 0 ? (static_cast<double>(suite.passed_tasks) / static_cast<double>(count)) * 100.0 : 0.0;
    suite.cost_per_resolved_task =
        suite.passed_tasks > 0 ? suite.total_cost_usd / static_cast<double>(suite.passed_tasks) : 0.0;

    print("\n\033[1m• BENCHMARK SUMMARY •\033[0m\n");
    print("  Passed:       {} / {} ({:.1f}%)\n", suite.passed_tasks, count, suite.pass_rate);
    print("  Duration:     {:.2f}s\n", suite.total_duration_secs);
    print("  Total Cost:   ${:.4f} USD\n", suite.total_cost_usd);
    print("  Cost/Resolve: ${:.4f} USD\n\n", suite.cost_per_resolved_task);

    std::string report_path = output_path;
    if (report_path.empty()) {
        make_dir("results");
        report_path = std::format("results/spacetime_results_{}.json", static_cast<long long>(now));
    }

    if (report_save_json(suite, report_path)) {
        print("\033[32mResults saved to {}\033[0m\n", report_path);
    } else {
        eprint("\033[31mFailed to write results to {}\033[0m\n", report_path);
    }

    return suite;
}

} // namespace spacetime
