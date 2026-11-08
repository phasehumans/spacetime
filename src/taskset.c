#define _GNU_SOURCE
#include "taskset.h"
#include "sha256.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static const char *TASKSET_FILES[4] = {"prompt.txt", "setup.sh", "test.sh", "meta.sh"};

static void file_digest(const char *dir, const char *file, char out[65]) {
    char path[1024];
    snprintf(path, sizeof(path), "%s/%s", dir, file);
    if (access(path, F_OK) != 0) {
        strcpy(out, "-");
        return;
    }
    if (sha256_file(path, out) != 0) {
        strcpy(out, "-");
    }
}

static int compare_strings(const void *a, const void *b) {
    const char *const *sa = (const char *const *)a;
    const char *const *sb = (const char *const *)b;
    return strcmp(*sa, *sb);
}

int taskset_manifest(const BenchmarkTask *tasks, size_t count, char **out_manifest) {
    if (!tasks || !out_manifest) return -1;

    char **rows = malloc(sizeof(char *) * count);
    if (!rows) return -1;

    for (size_t i = 0; i < count; i++) {
        const char *folder_name = strrchr(tasks[i].task_dir, '/');
        folder_name = folder_name ? folder_name + 1 : tasks[i].task_dir;
        if (!folder_name || *folder_name == '\0') {
            folder_name = tasks[i].id;
        }

        char d_prompt[65], d_setup[65], d_test[65], d_meta[65];
        file_digest(tasks[i].task_dir, TASKSET_FILES[0], d_prompt);
        file_digest(tasks[i].task_dir, TASKSET_FILES[1], d_setup);
        file_digest(tasks[i].task_dir, TASKSET_FILES[2], d_test);
        file_digest(tasks[i].task_dir, TASKSET_FILES[3], d_meta);

        char row[1024];
        snprintf(row, sizeof(row), "%s:%s:%s:%s:%s", folder_name, d_prompt, d_setup, d_test, d_meta);
        rows[i] = strdup(row);
    }

    qsort(rows, count, sizeof(char *), compare_strings);

    size_t total_len = 0;
    for (size_t i = 0; i < count; i++) {
        total_len += strlen(rows[i]) + 1; // row + '\n'
    }

    char *manifest = malloc(total_len + 1);
    if (!manifest) {
        for (size_t i = 0; i < count; i++) free(rows[i]);
        free(rows);
        return -1;
    }

    manifest[0] = '\0';
    for (size_t i = 0; i < count; i++) {
        strcat(manifest, rows[i]);
        strcat(manifest, "\n");
        free(rows[i]);
    }
    free(rows);

    *out_manifest = manifest;
    return 0;
}

int taskset_compute(const BenchmarkTask *tasks, size_t count, TasksetIdentity *out_identity) {
    if (!tasks || !out_identity) return -1;

    char *manifest = NULL;
    if (taskset_manifest(tasks, count, &manifest) != 0) {
        return -1;
    }

    char hash[65];
    sha256_bytes((const uint8_t *)manifest, strlen(manifest), hash);
    free(manifest);

    out_identity->size = count;
    strncpy(out_identity->hash, hash, sizeof(out_identity->hash) - 1);
    out_identity->hash[sizeof(out_identity->hash) - 1] = '\0';

    char short_hash[9];
    strncpy(short_hash, hash, 8);
    short_hash[8] = '\0';

    snprintf(out_identity->id, sizeof(out_identity->id), "%s-%zu-%s", TASKSET_SCHEME, count, short_hash);
    return 0;
}
