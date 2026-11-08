#ifndef SPACETIME_TASK_H
#define SPACETIME_TASK_H

#include "types.h"
#include <stddef.h>

int load_all_tasks(const char *tasks_dir, BenchmarkTask **out_tasks, size_t *out_count);
int find_task_by_id(const char *tasks_dir, const char *id_query, BenchmarkTask *out_task);
void free_tasks(BenchmarkTask *tasks, size_t count);

#endif /* SPACETIME_TASK_H */
