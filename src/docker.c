#define _GNU_SOURCE
#include "docker.h"
#include "../vendor/yyjson/yyjson.h"

#include <ctype.h>
#include <errno.h>
#include <fcntl.h>
#include <poll.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <sys/types.h>
#include <sys/un.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

#define MAX_ACTIVE_CONTAINERS 64
static char g_active_containers[MAX_ACTIVE_CONTAINERS][65];
static size_t g_active_count = 0;

void docker_register_active_container(const char *container_id) {
    if (!container_id || g_active_count >= MAX_ACTIVE_CONTAINERS) return;
    for (size_t i = 0; i < g_active_count; i++) {
        if (strcmp(g_active_containers[i], container_id) == 0) return;
    }
    strncpy(g_active_containers[g_active_count], container_id, 64);
    g_active_containers[g_active_count][64] = '\0';
    g_active_count++;
}

void docker_unregister_active_container(const char *container_id) {
    if (!container_id) return;
    for (size_t i = 0; i < g_active_count; i++) {
        if (strcmp(g_active_containers[i], container_id) == 0) {
            for (size_t j = i; j + 1 < g_active_count; j++) {
                strcpy(g_active_containers[j], g_active_containers[j + 1]);
            }
            g_active_count--;
            return;
        }
    }
}

int docker_connect(void) {
    int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0) return -1;

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, DOCKER_SOCK_PATH, sizeof(addr.sun_path) - 1);

    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        close(fd);
        return -1;
    }
    return fd;
}

void docker_response_free(DockerHttpResponse *resp) {
    if (resp && resp->body) {
        free(resp->body);
        resp->body = NULL;
        resp->body_len = 0;
    }
}

static int read_all(int fd, char **out_buf, size_t *out_len) {
    size_t cap = 8192;
    size_t len = 0;
    char *buf = malloc(cap);
    if (!buf) return -1;

    while (1) {
        if (len + 4096 > cap) {
            cap *= 2;
            char *new_buf = realloc(buf, cap);
            if (!new_buf) {
                free(buf);
                return -1;
            }
            buf = new_buf;
        }
        ssize_t n = read(fd, buf + len, cap - len - 1);
        if (n > 0) {
            len += (size_t)n;
        } else if (n == 0) {
            break;
        } else {
            if (errno == EINTR) continue;
            break;
        }
    }
    buf[len] = '\0';
    *out_buf = buf;
    *out_len = len;
    return 0;
}

int docker_request(const char *method, const char *path, const char *json_body, DockerHttpResponse *resp) {
    memset(resp, 0, sizeof(DockerHttpResponse));
    int fd = docker_connect();
    if (fd < 0) return -1;

    size_t body_len = json_body ? strlen(json_body) : 0;
    char header[2048];
    int header_len = snprintf(header, sizeof(header),
        "%s %s HTTP/1.1\r\n"
        "Host: localhost\r\n"
        "User-Agent: spacetime-c\r\n"
        "Accept: application/json\r\n"
        "Content-Type: application/json\r\n"
        "Content-Length: %zu\r\n"
        "Connection: close\r\n\r\n",
        method, path, body_len);

    if (write(fd, header, (size_t)header_len) < 0) {
        close(fd);
        return -1;
    }
    if (body_len > 0) {
        if (write(fd, json_body, body_len) < 0) {
            close(fd);
            return -1;
        }
    }

    char *raw = NULL;
    size_t raw_len = 0;
    if (read_all(fd, &raw, &raw_len) != 0 || raw_len == 0) {
        close(fd);
        if (raw) free(raw);
        return -1;
    }
    close(fd);

    // Parse HTTP Status
    if (strncmp(raw, "HTTP/1.", 7) == 0) {
        char *space = strchr(raw, ' ');
        if (space) {
            resp->status_code = atoi(space + 1);
        }
    }

    // Find end of headers (\r\n\r\n)
    char *body_start = strstr(raw, "\r\n\r\n");
    if (body_start) {
        body_start += 4;
        // Check if Transfer-Encoding: chunked
        if (strstr(raw, "Transfer-Encoding: chunked") || strstr(raw, "transfer-encoding: chunked")) {
            // Decode chunks
            size_t unchunked_cap = 4096;
            size_t unchunked_len = 0;
            char *unchunked = malloc(unchunked_cap);
            char *p = body_start;
            while (*p) {
                char *end_line = strstr(p, "\r\n");
                if (!end_line) break;
                long chunk_size = strtol(p, NULL, 16);
                if (chunk_size <= 0) break;
                p = end_line + 2;
                if (unchunked_len + (size_t)chunk_size + 1 > unchunked_cap) {
                    unchunked_cap = (unchunked_len + (size_t)chunk_size + 1) * 2;
                    unchunked = realloc(unchunked, unchunked_cap);
                }
                memcpy(unchunked + unchunked_len, p, (size_t)chunk_size);
                unchunked_len += (size_t)chunk_size;
                p += chunk_size;
                if (strncmp(p, "\r\n", 2) == 0) p += 2;
            }
            unchunked[unchunked_len] = '\0';
            resp->body = unchunked;
            resp->body_len = unchunked_len;
            free(raw);
        } else {
            size_t blen = raw_len - (size_t)(body_start - raw);
            resp->body = malloc(blen + 1);
            memcpy(resp->body, body_start, blen);
            resp->body[blen] = '\0';
            resp->body_len = blen;
            free(raw);
        }
    } else {
        resp->body = raw;
        resp->body_len = raw_len;
    }

    return 0;
}

int docker_check_daemon(void) {
    DockerHttpResponse resp;
    int err = docker_request("GET", "/version", NULL, &resp);
    if (err != 0 || resp.status_code != 200) {
        docker_response_free(&resp);
        return -1;
    }
    docker_response_free(&resp);
    return 0;
}

int docker_ensure_image(const char *image_tag, bool force_rebuild) {
    if (!force_rebuild) {
        char path[256];
        snprintf(path, sizeof(path), "/images/%s/json", image_tag);
        DockerHttpResponse resp;
        if (docker_request("GET", path, NULL, &resp) == 0 && resp.status_code == 200) {
            docker_response_free(&resp);
            return 0;
        }
        docker_response_free(&resp);
    }

    printf("\033[33mSandbox image '%s' not found or rebuild requested. Building...\033[0m\n", image_tag);

    const char *dockerfile = "Dockerfile";
    if (access("Dockerfile", F_OK) != 0 && access("Dockerfile.sandbox", F_OK) == 0) {
        dockerfile = "Dockerfile.sandbox";
    }

    pid_t pid = fork();
    if (pid == 0) {
        execlp("docker", "docker", "build", "-t", image_tag, "-f", dockerfile, ".", (char *)NULL);
        _exit(127);
    }
    int status;
    waitpid(pid, &status, 0);
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        fprintf(stderr, "\033[31mFailed to build Docker sandbox image '%s'\033[0m\n", image_tag);
        return -1;
    }
    printf("\033[32mSandbox image '%s' built successfully\033[0m\n", image_tag);
    return 0;
}

static void generate_container_name(char out[64]) {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    snprintf(out, 64, "spacetime-%lx-%04x", (unsigned long)tv.tv_sec, (unsigned int)(tv.tv_usec & 0xffff));
}

int docker_create_sandbox(const char *image, const char *mount_dir, char container_id_out[65]) {
    char name[64];
    generate_container_name(name);

    yyjson_mut_doc *doc = yyjson_mut_doc_new(NULL);
    yyjson_mut_val *root = yyjson_mut_obj(doc);
    yyjson_mut_doc_set_root(doc, root);

    yyjson_mut_obj_add_str(doc, root, "Image", image);
    yyjson_mut_obj_add_bool(doc, root, "Tty", true);
    yyjson_mut_obj_add_str(doc, root, "WorkingDir", "/home/agent");

    yyjson_mut_val *cmd = yyjson_mut_arr(doc);
    yyjson_mut_arr_add_str(doc, cmd, "/bin/sh");
    yyjson_mut_arr_add_str(doc, cmd, "-c");
    yyjson_mut_arr_add_str(doc, cmd, "sleep infinity");
    yyjson_mut_obj_add_val(doc, root, "Cmd", cmd);

    if (mount_dir && *mount_dir) {
        yyjson_mut_val *host_config = yyjson_mut_obj(doc);
        yyjson_mut_val *binds = yyjson_mut_arr(doc);
        char bind_str[1024];
        snprintf(bind_str, sizeof(bind_str), "%s:/workspace:rw", mount_dir);
        yyjson_mut_arr_add_str(doc, binds, bind_str);
        yyjson_mut_obj_add_val(doc, host_config, "Binds", binds);
        yyjson_mut_obj_add_val(doc, root, "HostConfig", host_config);
    }

    const char *json_body = yyjson_mut_write(doc, 0, NULL);

    char path[256];
    snprintf(path, sizeof(path), "/containers/create?name=%s", name);

    DockerHttpResponse resp;
    int err = docker_request("POST", path, json_body, &resp);
    free((void *)json_body);
    yyjson_mut_doc_free(doc);

    if (err != 0 || resp.status_code != 201) {
        fprintf(stderr, "Failed to create container (HTTP %d): %s\n", resp.status_code, resp.body ? resp.body : "");
        docker_response_free(&resp);
        return -1;
    }

    yyjson_doc *res_doc = yyjson_read(resp.body, resp.body_len, 0);
    if (!res_doc) {
        docker_response_free(&resp);
        return -1;
    }
    yyjson_val *id_val = yyjson_obj_get(yyjson_doc_get_root(res_doc), "Id");
    if (!id_val || !yyjson_is_str(id_val)) {
        yyjson_doc_free(res_doc);
        docker_response_free(&resp);
        return -1;
    }

    strncpy(container_id_out, yyjson_get_str(id_val), 64);
    container_id_out[64] = '\0';
    docker_register_active_container(container_id_out);

    yyjson_doc_free(res_doc);
    docker_response_free(&resp);
    return 0;
}

int docker_start_container(const char *container_id) {
    char path[256];
    snprintf(path, sizeof(path), "/containers/%s/start", container_id);
    DockerHttpResponse resp;
    int err = docker_request("POST", path, NULL, &resp);
    int status = resp.status_code;
    docker_response_free(&resp);
    return (err == 0 && (status == 204 || status == 304)) ? 0 : -1;
}

int docker_remove_container(const char *container_id, bool force) {
    char path[256];
    snprintf(path, sizeof(path), "/containers/%s?v=1&force=%s", container_id, force ? "true" : "false");
    DockerHttpResponse resp;
    int err = docker_request("DELETE", path, NULL, &resp);
    int status = resp.status_code;
    docker_response_free(&resp);
    docker_unregister_active_container(container_id);
    return (err == 0 && (status == 204 || status == 404)) ? 0 : -1;
}

int docker_terminate_agent_processes(const char *container_id) {
    ExecutionResult res;
    return docker_execute_script(container_id, "pkill -u agent -9 2>/dev/null || pkill -u 1000 -9 2>/dev/null || true", 5, &res);
}

void docker_cleanup_all_active_containers(void) {
    for (size_t i = 0; i < g_active_count; i++) {
        if (g_active_containers[i][0]) {
            docker_remove_container(g_active_containers[i], true);
        }
    }
    g_active_count = 0;
}

static int wait_exec_exit_code(const char *exec_id) {
    char path[256];
    snprintf(path, sizeof(path), "/exec/%s/json", exec_id);
    for (int retry = 0; retry < 100; retry++) {
        DockerHttpResponse resp;
        if (docker_request("GET", path, NULL, &resp) == 0 && resp.status_code == 200) {
            yyjson_doc *doc = yyjson_read(resp.body, resp.body_len, 0);
            if (doc) {
                yyjson_val *running = yyjson_obj_get(yyjson_doc_get_root(doc), "Running");
                if (running && !yyjson_get_bool(running)) {
                    yyjson_val *code = yyjson_obj_get(yyjson_doc_get_root(doc), "ExitCode");
                    int exit_code = code ? yyjson_get_int(code) : 0;
                    yyjson_doc_free(doc);
                    docker_response_free(&resp);
                    return exit_code;
                }
                yyjson_doc_free(doc);
            }
            docker_response_free(&resp);
        }
        usleep(50000); // 50ms
    }
    return -1;
}

// Executes a command and demultiplexes Docker's 8-byte framed stream (stdout / stderr)
static int run_exec_hijack(const char *exec_id, uint64_t timeout_secs, bool silent, ExecutionResult *res) {
    memset(res, 0, sizeof(ExecutionResult));
    int fd = docker_connect();
    if (fd < 0) return -1;

    yyjson_mut_doc *doc = yyjson_mut_doc_new(NULL);
    yyjson_mut_val *root = yyjson_mut_obj(doc);
    yyjson_mut_doc_set_root(doc, root);
    yyjson_mut_obj_add_bool(doc, root, "Detach", false);
    yyjson_mut_obj_add_bool(doc, root, "Tty", false);

    const char *body = yyjson_mut_write(doc, 0, NULL);
    char header[1024];
    snprintf(header, sizeof(header),
        "POST /exec/%s/start HTTP/1.1\r\n"
        "Host: localhost\r\n"
        "User-Agent: spacetime-c\r\n"
        "Content-Type: application/json\r\n"
        "Content-Length: %zu\r\n"
        "Connection: Upgrade\r\n"
        "Upgrade: tcp\r\n\r\n",
        exec_id, strlen(body));

    if (write(fd, header, strlen(header)) < 0 || write(fd, body, strlen(body)) < 0) {
        free((void *)body);
        yyjson_mut_doc_free(doc);
        close(fd);
        return -1;
    }
    free((void *)body);
    yyjson_mut_doc_free(doc);

    // Read HTTP response header (until \r\n\r\n)
    char http_buf[4096];
    size_t header_bytes = 0;
    while (header_bytes < sizeof(http_buf) - 1) {
        ssize_t n = read(fd, http_buf + header_bytes, 1);
        if (n <= 0) break;
        header_bytes++;
        http_buf[header_bytes] = '\0';
        if (strstr(http_buf, "\r\n\r\n")) break;
    }

    size_t stdout_cap = 8192, stdout_len = 0;
    char *stdout_buf = malloc(stdout_cap);
    size_t stderr_cap = 4096, stderr_len = 0;
    char *stderr_buf = malloc(stderr_cap);

    uint64_t start_time = (uint64_t)time(NULL);

    // Loop reading 8-byte multiplexed frames
    while (1) {
        struct pollfd pfd = {.fd = fd, .events = POLLIN, .revents = 0};
        int pr = poll(&pfd, 1, 100);
        if (pr < 0 && errno == EINTR) continue;

        uint64_t elapsed = (uint64_t)time(NULL) - start_time;
        if (timeout_secs > 0 && elapsed > timeout_secs) {
            res->timed_out = true;
            break;
        }

        if (pr == 0) continue; // poll timeout tick
        if (pfd.revents & (POLLERR | POLLHUP | POLLNVAL)) {
            // connection closed
            break;
        }

        if (pfd.revents & POLLIN) {
            uint8_t hdr[8];
            ssize_t hn = 0;
            while (hn < 8) {
                ssize_t r = read(fd, hdr + hn, (size_t)(8 - hn));
                if (r <= 0) break;
                hn += r;
            }
            if (hn < 8) break; // stream ended

            uint8_t stream_type = hdr[0]; // 1 = stdout, 2 = stderr
            uint32_t payload_len = ((uint32_t)hdr[4] << 24) |
                                   ((uint32_t)hdr[5] << 16) |
                                   ((uint32_t)hdr[6] << 8)  |
                                   ((uint32_t)hdr[7]);

            if (payload_len == 0) continue;

            char *payload = malloc(payload_len + 1);
            size_t pn = 0;
            while (pn < payload_len) {
                ssize_t r = read(fd, payload + pn, payload_len - pn);
                if (r <= 0) break;
                pn += (size_t)r;
            }
            payload[pn] = '\0';

            if (stream_type == 1) { // stdout
                if (!silent) {
                    fwrite(payload, 1, pn, stdout);
                    fflush(stdout);
                }
                if (stdout_len + pn + 1 > stdout_cap) {
                    stdout_cap = (stdout_len + pn + 1) * 2;
                    stdout_buf = realloc(stdout_buf, stdout_cap);
                }
                memcpy(stdout_buf + stdout_len, payload, pn);
                stdout_len += pn;
                stdout_buf[stdout_len] = '\0';
            } else if (stream_type == 2) { // stderr
                if (!silent) {
                    fwrite(payload, 1, pn, stderr);
                    fflush(stderr);
                }
                if (stderr_len + pn + 1 > stderr_cap) {
                    stderr_cap = (stderr_len + pn + 1) * 2;
                    stderr_buf = realloc(stderr_buf, stderr_cap);
                }
                memcpy(stderr_buf + stderr_len, payload, pn);
                stderr_len += pn;
                stderr_buf[stderr_len] = '\0';
            }
            free(payload);
        }
    }

    close(fd);

    res->stdout_buf = stdout_buf;
    res->stdout_len = stdout_len;
    res->stderr_buf = stderr_buf;
    res->stderr_len = stderr_len;

    res->exit_code = wait_exec_exit_code(exec_id);
    return 0;
}

int docker_execute_script(const char *container_id, const char *script_content, uint64_t timeout_secs, ExecutionResult *res) {
    char path[256];
    snprintf(path, sizeof(path), "/containers/%s/exec", container_id);

    yyjson_mut_doc *doc = yyjson_mut_doc_new(NULL);
    yyjson_mut_val *root = yyjson_mut_obj(doc);
    yyjson_mut_doc_set_root(doc, root);

    yyjson_mut_obj_add_bool(doc, root, "AttachStdout", true);
    yyjson_mut_obj_add_bool(doc, root, "AttachStderr", true);
    yyjson_mut_obj_add_str(doc, root, "User", "root");
    yyjson_mut_obj_add_str(doc, root, "WorkingDir", "/root");

    yyjson_mut_val *cmd = yyjson_mut_arr(doc);
    yyjson_mut_arr_add_str(doc, cmd, "/bin/bash");
    yyjson_mut_arr_add_str(doc, cmd, "-c");
    yyjson_mut_arr_add_str(doc, cmd, script_content);
    yyjson_mut_obj_add_val(doc, root, "Cmd", cmd);

    const char *json_body = yyjson_mut_write(doc, 0, NULL);
    DockerHttpResponse resp;
    int err = docker_request("POST", path, json_body, &resp);
    free((void *)json_body);
    yyjson_mut_doc_free(doc);

    if (err != 0 || resp.status_code != 201) {
        docker_response_free(&resp);
        return -1;
    }

    yyjson_doc *res_doc = yyjson_read(resp.body, resp.body_len, 0);
    if (!res_doc) {
        docker_response_free(&resp);
        return -1;
    }
    yyjson_val *id_val = yyjson_obj_get(yyjson_doc_get_root(res_doc), "Id");
    char exec_id[128];
    strncpy(exec_id, yyjson_get_str(id_val), sizeof(exec_id) - 1);
    exec_id[sizeof(exec_id) - 1] = '\0';
    yyjson_doc_free(res_doc);
    docker_response_free(&resp);

    return run_exec_hijack(exec_id, timeout_secs, true, res);
}

int docker_execute_agent(const char *container_id, const char *cmd, const char **env_vars, size_t env_count, uint64_t timeout_secs, bool silent, ExecutionResult *res) {
    char path[256];
    snprintf(path, sizeof(path), "/containers/%s/exec", container_id);

    yyjson_mut_doc *doc = yyjson_mut_doc_new(NULL);
    yyjson_mut_val *root = yyjson_mut_obj(doc);
    yyjson_mut_doc_set_root(doc, root);

    yyjson_mut_obj_add_bool(doc, root, "AttachStdout", true);
    yyjson_mut_obj_add_bool(doc, root, "AttachStderr", true);
    yyjson_mut_obj_add_str(doc, root, "User", "root");
    yyjson_mut_obj_add_str(doc, root, "WorkingDir", "/root");

    yyjson_mut_val *cmd_arr = yyjson_mut_arr(doc);
    yyjson_mut_arr_add_str(doc, cmd_arr, "/bin/bash");
    yyjson_mut_arr_add_str(doc, cmd_arr, "-c");
    yyjson_mut_arr_add_str(doc, cmd_arr, cmd);
    yyjson_mut_obj_add_val(doc, root, "Cmd", cmd_arr);

    if (env_vars && env_count > 0) {
        yyjson_mut_val *env_arr = yyjson_mut_arr(doc);
        for (size_t i = 0; i < env_count; i++) {
            yyjson_mut_arr_add_str(doc, env_arr, env_vars[i]);
        }
        yyjson_mut_obj_add_val(doc, root, "Env", env_arr);
    }

    const char *json_body = yyjson_mut_write(doc, 0, NULL);
    DockerHttpResponse resp;
    int err = docker_request("POST", path, json_body, &resp);
    free((void *)json_body);
    yyjson_mut_doc_free(doc);

    if (err != 0 || resp.status_code != 201) {
        docker_response_free(&resp);
        return -1;
    }

    yyjson_doc *res_doc = yyjson_read(resp.body, resp.body_len, 0);
    if (!res_doc) {
        docker_response_free(&resp);
        return -1;
    }
    yyjson_val *id_val = yyjson_obj_get(yyjson_doc_get_root(res_doc), "Id");
    char exec_id[128];
    strncpy(exec_id, yyjson_get_str(id_val), sizeof(exec_id) - 1);
    exec_id[sizeof(exec_id) - 1] = '\0';
    yyjson_doc_free(res_doc);
    docker_response_free(&resp);

    return run_exec_hijack(exec_id, timeout_secs, silent, res);
}

char *docker_scrub_secrets(const char *input) {
    if (!input) return strdup("");
    char *sanitized = strdup(input);
    const char *keys[] = {
        "ANTHROPIC_API_KEY", "GEMINI_API_KEY", "GOOGLE_API_KEY",
        "OPENAI_API_KEY", "DEEPSEEK_API_KEY", "GROQ_API_KEY",
        "OPENROUTER_API_KEY", "DATABRICKS_API_KEY", "DEVIN_API_KEY",
        "CURSOR_API_KEY", "INFLECTION_API_KEY", "HF_TOKEN"
    };
    for (size_t i = 0; i < sizeof(keys)/sizeof(keys[0]); i++) {
        const char *val = getenv(keys[i]);
        if (val && strlen(val) >= 4) {
            char *found;
            while ((found = strstr(sanitized, val)) != NULL) {
                memset(found, '*', strlen(val));
            }
        }
    }
    return sanitized;
}

void free_execution_result(ExecutionResult *res) {
    if (!res) return;
    if (res->stdout_buf) {
        free(res->stdout_buf);
        res->stdout_buf = NULL;
    }
    if (res->stderr_buf) {
        free(res->stderr_buf);
        res->stderr_buf = NULL;
    }
}

void free_task_result(TaskResult *res) {
    if (!res) return;
    if (res->error_message) {
        free(res->error_message);
        res->error_message = NULL;
    }
    if (res->agent_output) {
        free(res->agent_output);
        res->agent_output = NULL;
    }
}

void free_benchmark_suite_result(BenchmarkSuiteResult *suite) {
    if (!suite) return;
    if (suite->results) {
        for (size_t i = 0; i < suite->results_count; i++) {
            free_task_result(&suite->results[i]);
        }
        free(suite->results);
        suite->results = NULL;
    }
}
