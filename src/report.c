#define _GNU_SOURCE
#include "report.h"
#include "../vendor/yyjson/yyjson.h"

#include <dirent.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

int report_save_json(const BenchmarkSuiteResult *suite, const char *filepath) {
    yyjson_mut_doc *doc = yyjson_mut_doc_new(NULL);
    yyjson_mut_val *root = yyjson_mut_obj(doc);
    yyjson_mut_doc_set_root(doc, root);

    yyjson_mut_obj_add_str(doc, root, "timestamp", suite->timestamp);
    yyjson_mut_obj_add_str(doc, root, "agent", suite->agent);
    yyjson_mut_obj_add_str(doc, root, "harness", suite->harness);
    if (suite->model[0]) {
        yyjson_mut_obj_add_str(doc, root, "model", suite->model);
    } else {
        yyjson_mut_obj_add_null(doc, root, "model");
    }
    yyjson_mut_obj_add_str(doc, root, "spacetime_version", suite->spacetime_version);
    yyjson_mut_obj_add_str(doc, root, "taskset_id", suite->taskset_id);
    yyjson_mut_obj_add_str(doc, root, "taskset_hash", suite->taskset_hash);
    yyjson_mut_obj_add_uint(doc, root, "taskset_size", suite->taskset_size);
    yyjson_mut_obj_add_str(doc, root, "sandbox", suite->sandbox);
    yyjson_mut_obj_add_uint(doc, root, "total_tasks", suite->total_tasks);
    yyjson_mut_obj_add_uint(doc, root, "passed_tasks", suite->passed_tasks);
    yyjson_mut_obj_add_uint(doc, root, "failed_tasks", suite->failed_tasks);
    yyjson_mut_obj_add_real(doc, root, "pass_rate", suite->pass_rate);
    yyjson_mut_obj_add_real(doc, root, "total_duration_secs", suite->total_duration_secs);
    yyjson_mut_obj_add_uint(doc, root, "total_tokens", suite->total_tokens);
    yyjson_mut_obj_add_real(doc, root, "total_cost_usd", suite->total_cost_usd);
    yyjson_mut_obj_add_real(doc, root, "cost_per_resolved_task", suite->cost_per_resolved_task);

    yyjson_mut_val *results_arr = yyjson_mut_arr(doc);
    for (size_t i = 0; i < suite->results_count; i++) {
        const TaskResult *r = &suite->results[i];
        yyjson_mut_val *item = yyjson_mut_obj(doc);
        yyjson_mut_obj_add_str(doc, item, "task_id", r->task_id);
        yyjson_mut_obj_add_str(doc, item, "task_name", r->task_name);
        yyjson_mut_obj_add_bool(doc, item, "passed", r->passed);
        yyjson_mut_obj_add_real(doc, item, "duration_secs", r->duration_secs);
        yyjson_mut_obj_add_str(doc, item, "agent_name", r->agent_name);
        if (r->has_exit_code) {
            yyjson_mut_obj_add_int(doc, item, "exit_code", r->exit_code);
        } else {
            yyjson_mut_obj_add_null(doc, item, "exit_code");
        }
        if (r->error_message) {
            yyjson_mut_obj_add_str(doc, item, "error_message", r->error_message);
        } else {
            yyjson_mut_obj_add_null(doc, item, "error_message");
        }
        yyjson_mut_obj_add_str(doc, item, "agent_output", r->agent_output ? r->agent_output : "");
        yyjson_mut_obj_add_uint(doc, item, "prompt_tokens", r->prompt_tokens);
        yyjson_mut_obj_add_uint(doc, item, "completion_tokens", r->completion_tokens);
        yyjson_mut_obj_add_uint(doc, item, "total_tokens", r->total_tokens);
        yyjson_mut_obj_add_real(doc, item, "estimated_cost_usd", r->estimated_cost_usd);
        yyjson_mut_arr_add_val(results_arr, item);
    }
    yyjson_mut_obj_add_val(doc, root, "results", results_arr);

    yyjson_write_flag flg = YYJSON_WRITE_PRETTY;
    size_t json_len = 0;
    char *json = yyjson_mut_write(doc, flg, &json_len);
    yyjson_mut_doc_free(doc);

    if (!json) return -1;

    FILE *f = fopen(filepath, "wb");
    if (!f) {
        free(json);
        return -1;
    }
    fwrite(json, 1, json_len, f);
    fclose(f);
    free(json);
    return 0;
}

int submit_create(const char *report_path, const char *user, const char *tier, const char *source, const char *notes, const char *out_dir) {
    if (!report_path || access(report_path, R_OK) != 0) {
        fprintf(stderr, "Report file not found or not readable: %s\n", report_path ? report_path : "");
        return -1;
    }

    yyjson_doc *doc = yyjson_read_file(report_path, 0, NULL, NULL);
    if (!doc) {
        fprintf(stderr, "Failed to parse report JSON.\n");
        return -1;
    }

    yyjson_val *root = yyjson_doc_get_root(doc);
    const char *taskset_id = yyjson_get_str(yyjson_obj_get(root, "taskset_id"));
    if (!taskset_id || !*taskset_id) {
        taskset_id = "v1-default";
    }

    char target_dir[512];
    snprintf(target_dir, sizeof(target_dir), "%s/%s", out_dir ? out_dir : DEFAULT_SUBMISSIONS_DIR, taskset_id);

    char mkdir_cmd[1024];
    snprintf(mkdir_cmd, sizeof(mkdir_cmd), "mkdir -p %s", target_dir);
    int sys_rc = system(mkdir_cmd);
    (void)sys_rc;

    time_t now = time(NULL);
    char submission_file[1024];
    const char *username = (user && *user) ? user : "anonymous";
    snprintf(submission_file, sizeof(submission_file), "%s/%s_%ld.json", target_dir, username, (long)now);

    yyjson_mut_doc *sub_doc = yyjson_doc_mut_copy(doc, NULL);
    yyjson_doc_free(doc);

    yyjson_mut_val *sub_root = yyjson_mut_doc_get_root(sub_doc);
    yyjson_mut_obj_add_str(sub_doc, sub_root, "user", username);
    yyjson_mut_obj_add_str(sub_doc, sub_root, "tier", tier ? tier : "community");
    if (source && *source) yyjson_mut_obj_add_str(sub_doc, sub_root, "source", source);
    if (notes && *notes) yyjson_mut_obj_add_str(sub_doc, sub_root, "notes", notes);

    size_t out_len = 0;
    char *out_json = yyjson_mut_write(sub_doc, YYJSON_WRITE_PRETTY, &out_len);
    yyjson_mut_doc_free(sub_doc);

    FILE *f = fopen(submission_file, "wb");
    if (!f) {
        free(out_json);
        return -1;
    }
    fwrite(out_json, 1, out_len, f);
    fclose(f);
    free(out_json);

    printf("\033[32mSubmission saved to %s\033[0m\n", submission_file);
    return 0;
}

int leaderboard_render(const char *submissions_dir, bool json_mode, const char *taskset_filter) {
    DIR *d = opendir(submissions_dir ? submissions_dir : DEFAULT_SUBMISSIONS_DIR);
    if (!d) {
        printf("No submissions found under %s\n", submissions_dir ? submissions_dir : DEFAULT_SUBMISSIONS_DIR);
        return 0;
    }

    if (!json_mode) {
        printf("\n\033[1;37m• SPACETIME GLOBAL LEADERBOARD •\033[0m\n");
        printf("%-16s %-20s %-10s %-12s %-10s %-8s\n",
            "USER", "AGENT", "PASS RATE", "COST/RESOLV", "AVG TIME", "TIER");
        printf("--------------------------------------------------------------------------------\n");
    }

    struct dirent *dir;
    while ((dir = readdir(d)) != NULL) {
        if (dir->d_name[0] == '.') continue;
        if (taskset_filter && strcmp(dir->d_name, taskset_filter) != 0) continue;

        char subdir_path[1024];
        snprintf(subdir_path, sizeof(subdir_path), "%s/%s",
            submissions_dir ? submissions_dir : DEFAULT_SUBMISSIONS_DIR, dir->d_name);

        DIR *sub = opendir(subdir_path);
        if (!sub) continue;
        struct dirent *sub_ent;
        while ((sub_ent = readdir(sub)) != NULL) {
            if (sub_ent->d_name[0] == '.' || !strstr(sub_ent->d_name, ".json")) continue;
            char file_path[2048];
            snprintf(file_path, sizeof(file_path), "%s/%s", subdir_path, sub_ent->d_name);
            yyjson_doc *doc = yyjson_read_file(file_path, 0, NULL, NULL);
            if (!doc) continue;
            yyjson_val *r = yyjson_doc_get_root(doc);

            const char *u = yyjson_get_str(yyjson_obj_get(r, "user"));
            const char *ag = yyjson_get_str(yyjson_obj_get(r, "agent"));
            double pr = yyjson_get_num(yyjson_obj_get(r, "pass_rate"));
            double cpr = yyjson_get_num(yyjson_obj_get(r, "cost_per_resolved_task"));
            double dur = yyjson_get_num(yyjson_obj_get(r, "total_duration_secs"));
            const char *tier = yyjson_get_str(yyjson_obj_get(r, "tier"));

            if (!json_mode) {
                printf("%-16s %-20s %6.1f%%    $%6.4f     %6.1fs   %-8s\n",
                    u ? u : "-", ag ? ag : "-", pr, cpr, dur, tier ? tier : "community");
            }
            yyjson_doc_free(doc);
        }
        closedir(sub);
    }
    closedir(d);
    if (!json_mode) {
        printf("--------------------------------------------------------------------------------\n\n");
    }
    return 0;
}
