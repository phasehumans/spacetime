#include "catch2/catch_amalgamated.hpp"

#include <string>

#include "nlohmann/json.hpp"
#include "report.hpp"
#include "test_helpers.hpp"
#include "util.hpp"

using namespace spacetime;
using nlohmann::ordered_json;

namespace {

BenchmarkSuiteResult sample_suite() {
    BenchmarkSuiteResult suite;
    suite.timestamp = "2026-01-02T03:04:05Z";
    suite.agent = "Claude Code";
    suite.harness = "Claude Code";
    suite.model = "some-model";
    suite.spacetime_version = kVersion;
    suite.taskset_id = "v1-2-deadbeef";
    suite.taskset_hash = std::string(64, 'd');
    suite.taskset_size = 2;
    suite.sandbox = kDefaultSandboxImage;
    suite.total_tasks = 2;
    suite.passed_tasks = 1;
    suite.failed_tasks = 1;
    suite.pass_rate = 50.0;
    suite.total_duration_secs = 12.5;
    suite.total_tokens = 900;
    suite.total_cost_usd = 0.0123;
    suite.cost_per_resolved_task = 0.0123;

    TaskResult pass;
    pass.task_id = "001-a";
    pass.task_name = "Task A";
    pass.passed = true;
    pass.duration_secs = 5.25;
    pass.agent_name = "Claude Code";
    pass.exit_code = 0;
    pass.agent_output = "done \"quoted\"\n\ttabbed ✓";
    pass.prompt_tokens = 400;
    pass.completion_tokens = 50;
    pass.total_tokens = 450;
    pass.estimated_cost_usd = 0.00195;

    TaskResult fail;
    fail.task_id = "002-b";
    fail.task_name = "Task B";
    fail.agent_name = "Claude Code";
    fail.error_message = "Failed to create Docker sandbox container";

    suite.results = {pass, fail};
    return suite;
}

} // namespace

TEST_CASE("the report serializes every suite field", "[report]") {
    const ordered_json doc = ordered_json::parse(report_to_json(sample_suite()));

    CHECK(doc["timestamp"] == "2026-01-02T03:04:05Z");
    CHECK(doc["agent"] == "Claude Code");
    CHECK(doc["harness"] == "Claude Code");
    CHECK(doc["model"] == "some-model");
    CHECK(doc["spacetime_version"] == kVersion);
    CHECK(doc["taskset_id"] == "v1-2-deadbeef");
    CHECK(doc["taskset_size"] == 2);
    CHECK(doc["total_tasks"] == 2);
    CHECK(doc["passed_tasks"] == 1);
    CHECK(doc["failed_tasks"] == 1);
    CHECK(doc["pass_rate"] == 50.0);
    CHECK(doc["total_duration_secs"] == 12.5);
    CHECK(doc["total_tokens"] == 900);
    CHECK(doc["total_cost_usd"] == 0.0123);

    REQUIRE(doc["results"].size() == 2);
    const auto& pass = doc["results"][0];
    CHECK(pass["task_id"] == "001-a");
    CHECK(pass["passed"] == true);
    CHECK(pass["exit_code"] == 0);
    CHECK(pass["error_message"].is_null());
    CHECK(pass["agent_output"] == "done \"quoted\"\n\ttabbed ✓");
    CHECK(pass["estimated_cost_usd"] == 0.00195);

    const auto& fail = doc["results"][1];
    CHECK(fail["passed"] == false);
    CHECK(fail["exit_code"].is_null());
    CHECK(fail["error_message"] == "Failed to create Docker sandbox container");
    CHECK(fail["agent_output"] == "");
}

TEST_CASE("report keys keep their documented order", "[report]") {
    const ordered_json doc = ordered_json::parse(report_to_json(sample_suite()));

    std::vector<std::string> keys;
    for (const auto& item : doc.items()) keys.push_back(item.key());
    CHECK(keys == std::vector<std::string>{
                      "timestamp", "agent", "harness", "model", "spacetime_version", "taskset_id",
                      "taskset_hash", "taskset_size", "sandbox", "total_tasks", "passed_tasks",
                      "failed_tasks", "pass_rate", "total_duration_secs", "total_tokens",
                      "total_cost_usd", "cost_per_resolved_task", "results"});

    std::vector<std::string> result_keys;
    for (const auto& item : doc["results"][0].items()) result_keys.push_back(item.key());
    CHECK(result_keys == std::vector<std::string>{
                             "task_id", "task_name", "passed", "duration_secs", "agent_name",
                             "exit_code", "error_message", "agent_output", "prompt_tokens",
                             "completion_tokens", "total_tokens", "estimated_cost_usd"});
}

TEST_CASE("an empty model is reported as null", "[report]") {
    BenchmarkSuiteResult suite = sample_suite();
    suite.model.clear();
    suite.results.clear();

    const ordered_json doc = ordered_json::parse(report_to_json(suite));
    CHECK(doc["model"].is_null());
    CHECK(doc["results"].is_array());
    CHECK(doc["results"].empty());
}

TEST_CASE("agent output with invalid UTF-8 still produces a valid report", "[report]") {
    BenchmarkSuiteResult suite = sample_suite();
    suite.results[0].agent_output = "before \xff\xfe after";

    std::string text;
    REQUIRE_NOTHROW(text = report_to_json(suite));
    const ordered_json doc = ordered_json::parse(text);
    CHECK(doc["results"][0]["agent_output"] == "before \xEF\xBF\xBD\xEF\xBF\xBD after");
}

TEST_CASE("report_save_json writes the report to disk", "[report]") {
    TempDir dir;
    const BenchmarkSuiteResult suite = sample_suite();
    const std::string path = dir.path() + "/report.json";

    REQUIRE(report_save_json(suite, path));
    CHECK(read_file(path) == report_to_json(suite));

    CHECK_FALSE(report_save_json(suite, dir.path() + "/missing-dir/report.json"));
}
