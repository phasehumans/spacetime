#ifndef SPACETIME_EVAL_H
#define SPACETIME_EVAL_H

#include "types.h"
#include <stdbool.h>
#include <stddef.h>

int eval_run_task(const BenchmarkTask *task, const AgentProfile *agent, const char *image, uint64_t timeout_override, bool silent, TaskResult *out_res);
int eval_run_suite(const BenchmarkTask *tasks, size_t count, const AgentProfile *agent, const char *image, uint64_t timeout_override, const char *output_path, BenchmarkSuiteResult *out_suite);

#endif /* SPACETIME_EVAL_H */
