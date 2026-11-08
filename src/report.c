#define _GNU_SOURCE
#include "report.h"
#include "../vendor/yyjson/yyjson.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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
