#ifndef SPACETIME_TASKSET_H
#define SPACETIME_TASKSET_H

#include "types.h"
#include <stddef.h>

#define TASKSET_SCHEME "v1"

int taskset_compute(const BenchmarkTask *tasks, size_t count, TasksetIdentity *out_identity);
int taskset_manifest(const BenchmarkTask *tasks, size_t count, char **out_manifest);

#endif /* SPACETIME_TASKSET_H */
