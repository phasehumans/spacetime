#define _GNU_SOURCE
#include "task.h"
#include <ctype.h>
#include <dirent.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>
#include <sys/stat.h>
#include <unistd.h>

static char *trim_whitespace(char *str) {
    if (!str) return NULL;
    while (isspace((unsigned char)*str)) str++;
    if (*str == 0) return str;
    char *end = str + strlen(str) - 1;
    while (end > str && isspace((unsigned char)*end)) end--;
    end[1] = '\0';
    return str;
}

static char *read_file_to_string(const char *path) {
    FILE *f = fopen(path, "rb");
    if (!f) return NULL;

    fseek(f, 0, SEEK_END);
    long sz = ftell(f);
    fseek(f, 0, SEEK_SET);

    if (sz < 0) {
        fclose(f);
        return NULL;
    }

    char *buf = malloc((size_t)sz + 1);
    if (!buf) {
        fclose(f);
        return NULL;
    }

    size_t read_bytes = fread(buf, 1, (size_t)sz, f);
    buf[read_bytes] = '\0';
    fclose(f);
    return buf;
}

static int compare_tasks(const void *a, const void *b) {
    const BenchmarkTask *ta = (const BenchmarkTask *)a;
    const BenchmarkTask *tb = (const BenchmarkTask *)b;
    return strcmp(ta->id, tb->id);
}

static int load_task_from_dir(const char *dir_path, BenchmarkTask *task) {
    memset(task, 0, sizeof(BenchmarkTask));

    const char *folder_name = strrchr(dir_path, '/');
    folder_name = folder_name ? folder_name + 1 : dir_path;

    snprintf(task->id, sizeof(task->id), "%s", folder_name);
    snprintf(task->name, sizeof(task->name), "%s", folder_name);
    snprintf(task->base_image, sizeof(task->base_image), "%s", "ubuntu:22.04");
    snprintf(task->task_dir, sizeof(task->task_dir), "%s", dir_path);
    task->max_turns = 15;
    task->timeout_secs = 30;

    snprintf(task->setup_script, sizeof(task->setup_script), "%.400s/setup.sh", dir_path);
    snprintf(task->test_script, sizeof(task->test_script), "%.400s/test.sh", dir_path);

    char prompt_path[2048];
    snprintf(prompt_path, sizeof(prompt_path), "%.1000s/prompt.txt", dir_path);
    char *prompt_raw = read_file_to_string(prompt_path);
    if (prompt_raw) {
        char *trimmed = trim_whitespace(prompt_raw);
        task->prompt = strdup(trimmed);
        free(prompt_raw);
    } else {
        task->prompt = strdup("");
    }

    char meta_path[2048];
    snprintf(meta_path, sizeof(meta_path), "%.1000s/meta.sh", dir_path);
    char *meta_content = read_file_to_string(meta_path);
    if (!meta_content) {
        snprintf(meta_path, sizeof(meta_path), "%.1000s/meta.toml", dir_path);
        meta_content = read_file_to_string(meta_path);
    }

    if (meta_content) {
        char *line = strtok(meta_content, "\r\n");
        while (line) {
            line = trim_whitespace(line);
            if (*line != '#' && *line != '\0') {
                char *eq = strchr(line, '=');
                if (eq) {
                    *eq = '\0';
                    char *key = trim_whitespace(line);
                    char *val = trim_whitespace(eq + 1);

                    // strip surrounding quotes
                    if ((*val == '"' || *val == '\'') && val[strlen(val) - 1] == *val) {
                        val[strlen(val) - 1] = '\0';
                        val++;
                    }

                    if (strcasecmp(key, "TASK_ID") == 0 || strcasecmp(key, "ID") == 0) {
                        snprintf(task->id, sizeof(task->id), "%s", val);
                    } else if (strcasecmp(key, "TASK_NAME") == 0 || strcasecmp(key, "NAME") == 0) {
                        snprintf(task->name, sizeof(task->name), "%s", val);
                    } else if (strcasecmp(key, "BASE_IMAGE") == 0 || strcasecmp(key, "IMAGE") == 0) {
                        snprintf(task->base_image, sizeof(task->base_image), "%s", val);
                    } else if (strcasecmp(key, "MAX_TURNS") == 0) {
                        task->max_turns = (uint32_t)strtoul(val, NULL, 10);
                    } else if (strcasecmp(key, "TIMEOUT_SECS") == 0 || strcasecmp(key, "TIMEOUT") == 0) {
                        task->timeout_secs = strtoull(val, NULL, 10);
                    } else if (strcasecmp(key, "DESCRIPTION") == 0 || strcasecmp(key, "DESC") == 0) {
                        snprintf(task->description, sizeof(task->description), "%s", val);
                    }
                }
            }
            line = strtok(NULL, "\r\n");
        }
        free(meta_content);
    }

    if (access(task->setup_script, F_OK) != 0) {
        if (task->prompt) free(task->prompt);
        return -1;
    }
    if (access(task->test_script, F_OK) != 0) {
        if (task->prompt) free(task->prompt);
        return -1;
    }

    return 0;
}

int load_all_tasks(const char *tasks_dir, BenchmarkTask **out_tasks, size_t *out_count) {
    if (!tasks_dir || !out_tasks || !out_count) return -1;

    DIR *d = opendir(tasks_dir);
    if (!d) return -1;

    size_t capacity = 64;
    size_t count = 0;
    BenchmarkTask *tasks = malloc(sizeof(BenchmarkTask) * capacity);
    if (!tasks) {
        closedir(d);
        return -1;
    }

    struct dirent *dir;
    while ((dir = readdir(d)) != NULL) {
        if (strcmp(dir->d_name, ".") == 0 || strcmp(dir->d_name, "..") == 0) {
            continue;
        }

        char path[1024];
        snprintf(path, sizeof(path), "%s/%s", tasks_dir, dir->d_name);

        struct stat st;
        if (stat(path, &st) == 0 && S_ISDIR(st.st_mode)) {
            char check_prompt[2048];
            snprintf(check_prompt, sizeof(check_prompt), "%s/prompt.txt", path);
            char check_meta[2048];
            snprintf(check_meta, sizeof(check_meta), "%s/meta.sh", path);

            if (access(check_prompt, F_OK) == 0 || access(check_meta, F_OK) == 0) {
                if (count >= capacity) {
                    capacity *= 2;
                    BenchmarkTask *resized = realloc(tasks, sizeof(BenchmarkTask) * capacity);
                    if (!resized) {
                        free_tasks(tasks, count);
                        closedir(d);
                        return -1;
                    }
                    tasks = resized;
                }

                if (load_task_from_dir(path, &tasks[count]) == 0) {
                    count++;
                }
            }
        }
    }
    closedir(d);

    qsort(tasks, count, sizeof(BenchmarkTask), compare_tasks);

    *out_tasks = tasks;
    *out_count = count;
    return 0;
}

int find_task_by_id(const char *tasks_dir, const char *id_query, BenchmarkTask *out_task) {
    BenchmarkTask *tasks = NULL;
    size_t count = 0;
    if (load_all_tasks(tasks_dir, &tasks, &count) != 0 || count == 0) {
        return -1;
    }

    int found_idx = -1;
    // 1. Exact match
    for (size_t i = 0; i < count; i++) {
        if (strcmp(tasks[i].id, id_query) == 0) {
            found_idx = (int)i;
            break;
        }
    }

    // 2. Prefix or substring match
    if (found_idx < 0) {
        const char *clean_query = id_query;
        if (strncmp(clean_query, "task-", 5) == 0) clean_query += 5;

        for (size_t i = 0; i < count; i++) {
            if (strstr(tasks[i].id, id_query) ||
                strstr(tasks[i].id, clean_query) ||
                strcasestr(tasks[i].name, id_query)) {
                found_idx = (int)i;
                break;
            }
        }
    }

    if (found_idx >= 0) {
        memcpy(out_task, &tasks[found_idx], sizeof(BenchmarkTask));
        out_task->prompt = strdup(tasks[found_idx].prompt);
        free_tasks(tasks, count);
        return 0;
    }

    free_tasks(tasks, count);
    return -1;
}

void free_tasks(BenchmarkTask *tasks, size_t count) {
    if (!tasks) return;
    for (size_t i = 0; i < count; i++) {
        if (tasks[i].prompt) {
            free(tasks[i].prompt);
            tasks[i].prompt = NULL;
        }
    }
    free(tasks);
}
