#include "report.hpp"

#include <fstream>
#include <optional>

#include "nlohmann/json.hpp"

namespace spacetime {

namespace {

// Keeps keys in insertion order so the report reads top-down.
using json = nlohmann::ordered_json;

template <typename T>
json nullable(const std::optional<T>& value) {
    return value ? json(*value) : json(nullptr);
}

} // namespace

std::string report_to_json(const BenchmarkSuiteResult& suite) {
    json root;
    root["timestamp"] = suite.timestamp;
    root["agent"] = suite.agent;
    root["harness"] = suite.harness;
    root["model"] = suite.model.empty() ? json(nullptr) : json(suite.model);
    root["spacetime_version"] = suite.spacetime_version;
    root["taskset_id"] = suite.taskset_id;
    root["taskset_hash"] = suite.taskset_hash;
    root["taskset_size"] = suite.taskset_size;
    root["sandbox"] = suite.sandbox;
    root["total_tasks"] = suite.total_tasks;
    root["passed_tasks"] = suite.passed_tasks;
    root["failed_tasks"] = suite.failed_tasks;
    root["pass_rate"] = suite.pass_rate;
    root["total_duration_secs"] = suite.total_duration_secs;
    root["total_tokens"] = suite.total_tokens;
    root["total_cost_usd"] = suite.total_cost_usd;
    root["cost_per_resolved_task"] = suite.cost_per_resolved_task;

    json results = json::array();
    for (const TaskResult& r : suite.results) {
        json item;
        item["task_id"] = r.task_id;
        item["task_name"] = r.task_name;
        item["passed"] = r.passed;
        item["duration_secs"] = r.duration_secs;
        item["agent_name"] = r.agent_name;
        item["exit_code"] = nullable(r.exit_code);
        item["error_message"] = nullable(r.error_message);
        item["agent_output"] = r.agent_output;
        item["prompt_tokens"] = r.prompt_tokens;
        item["completion_tokens"] = r.completion_tokens;
        item["total_tokens"] = r.total_tokens;
        item["estimated_cost_usd"] = r.estimated_cost_usd;
        results.push_back(std::move(item));
    }
    root["results"] = std::move(results);

    // Agent output is arbitrary bytes; replace invalid UTF-8 instead of throwing.
    return root.dump(4, ' ', false, json::error_handler_t::replace);
}

bool report_save_json(const BenchmarkSuiteResult& suite, const std::string& filepath) {
    const std::string text = report_to_json(suite);

    std::ofstream out(filepath, std::ios::binary);
    if (!out) return false;
    out.write(text.data(), static_cast<std::streamsize>(text.size()));
    out.flush();
    return out.good();
}

} // namespace spacetime
