#ifndef SPACETIME_DOCKER_H
#define SPACETIME_DOCKER_H

#include "types.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define DOCKER_SOCK_PATH "/var/run/docker.sock"

typedef struct {
    int status_code;
    char *body;
    size_t body_len;
} DockerHttpResponse;

int docker_connect(void);
void docker_response_free(DockerHttpResponse *resp);
int docker_request(const char *method, const char *path, const char *json_body, DockerHttpResponse *resp);

int docker_check_daemon(void);
int docker_ensure_image(const char *image_tag, bool force_rebuild);

int docker_create_sandbox(const char *image, const char *mount_dir, char container_id_out[65]);
int docker_start_container(const char *container_id);
int docker_remove_container(const char *container_id, bool force);
int docker_terminate_agent_processes(const char *container_id);

int docker_execute_script(const char *container_id, const char *script_content, uint64_t timeout_secs, ExecutionResult *res);
int docker_execute_agent(const char *container_id, const char *cmd, const char **env_vars, size_t env_count, uint64_t timeout_secs, bool silent, ExecutionResult *res);

void docker_register_active_container(const char *container_id);
void docker_unregister_active_container(const char *container_id);
void docker_cleanup_all_active_containers(void);

char *docker_scrub_secrets(const char *input);

#endif /* SPACETIME_DOCKER_H */
