#include <stdio.h>
#include <string.h>
#include <assert.h>
#include "docker.h"

int main(void) {
    printf("[TEST] Checking Docker daemon...\n");
    int err = docker_check_daemon();
    if (err != 0) {
        fprintf(stderr, "Docker daemon not reachable!\n");
        return 1;
    }
    printf("[TEST] Docker daemon reachable!\n");

    printf("[TEST] Creating test container...\n");
    char container_id[65];
    err = docker_create_sandbox("ubuntu:24.04", NULL, container_id);
    if (err != 0) {
        fprintf(stderr, "Failed to create container!\n");
        return 1;
    }
    printf("[TEST] Created container: %s\n", container_id);

    printf("[TEST] Starting container...\n");
    err = docker_start_container(container_id);
    if (err != 0) {
        fprintf(stderr, "Failed to start container!\n");
        docker_remove_container(container_id, true);
        return 1;
    }
    printf("[TEST] Container started successfully.\n");

    printf("[TEST] Executing script inside container...\n");
    ExecutionResult res;
    err = docker_execute_script(container_id, "echo 'hello from spacetime-c' && whoami", 10, &res);
    if (err != 0) {
        fprintf(stderr, "Failed to execute script!\n");
        docker_remove_container(container_id, true);
        return 1;
    }
    printf("[TEST] Exit code: %d\n", res.exit_code);
    printf("[TEST] Stdout: %s", res.stdout_buf ? res.stdout_buf : "<null>");
    assert(res.exit_code == 0);
    assert(strstr(res.stdout_buf, "hello from spacetime-c") != NULL);
    assert(strstr(res.stdout_buf, "root") != NULL);

    free_execution_result(&res);

    printf("[TEST] Removing container...\n");
    err = docker_remove_container(container_id, true);
    if (err != 0) {
        fprintf(stderr, "Failed to remove container!\n");
        return 1;
    }
    printf("[TEST] Container removed cleanly.\n");

    printf("[TEST] PASS: Docker AF_UNIX client operates with 100%% success!\n");
    return 0;
}
