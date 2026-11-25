#pragma once

#include <cstdint>
#include <string>
#include <vector>

#include "types.hpp"

namespace spacetime {

// Runs one task end to end: sandbox, setup.sh, agent, test.sh, teardown. A
// `timeout_override` of 0 uses the task's own timeout. Infrastructure failures
// are reported through TaskResult::error_message.
TaskResult eval_run_task(const BenchmarkTask& task, const AgentProfile& agent, const std::string& image,
                         std::uint64_t timeout_override, bool silent);

// Runs every task in order and writes the JSON report. An empty `output_path`
// writes to results/spacetime_results_<unix-time>.json.
BenchmarkSuiteResult eval_run_suite(const std::vector<BenchmarkTask>& tasks, const AgentProfile& agent,
                                    const std::string& image, std::uint64_t timeout_override,
                                    const std::string& output_path = {});

} // namespace spacetime
