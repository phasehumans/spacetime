#include <stdio.h>
#include <string.h>
#include <assert.h>
#include "task.h"
#include "taskset.h"

int main(void) {
    printf("[TEST] Loading tasks from tasks/...\n");
    BenchmarkTask *tasks = NULL;
    size_t count = 0;
    int err = load_all_tasks("tasks", &tasks, &count);
    if (err != 0 || count == 0) {
        fprintf(stderr, "Failed to load tasks! err=%d, count=%zu\n", err, count);
        return 1;
    }
    printf("[TEST] Loaded %zu tasks.\n", count);
    assert(count == 50);

    TasksetIdentity id;
    err = taskset_compute(tasks, count, &id);
    if (err != 0) {
        fprintf(stderr, "Failed to compute taskset identity!\n");
        return 1;
    }

    printf("[TEST] Taskset ID:   %s\n", id.id);
    printf("[TEST] Taskset Hash: %s\n", id.hash);
    printf("[TEST] Taskset Size: %zu\n", id.size);

    const char *expected_hash = "7ecd749976c413ce1f51daf4dd09ca9e987548468c4bcf40430aad44582f3f16";
    const char *expected_id = "v1-50-7ecd7499";

    if (strcmp(id.hash, expected_hash) != 0) {
        fprintf(stderr, "HASH MISMATCH! Expected %s, got %s\n", expected_hash, id.hash);
        return 1;
    }
    if (strcmp(id.id, expected_id) != 0) {
        fprintf(stderr, "ID MISMATCH! Expected %s, got %s\n", expected_id, id.id);
        return 1;
    }

    free_tasks(tasks, count);
    printf("[TEST] PASS: taskset digest matches validator perfectly!\n");
    return 0;
}
