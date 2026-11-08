#ifndef SPACETIME_REPORT_H
#define SPACETIME_REPORT_H

#include "types.h"
#include <stdbool.h>

int report_save_json(const BenchmarkSuiteResult *suite, const char *filepath);
int submit_create(const char *report_path, const char *user, const char *tier, const char *source, const char *notes, const char *out_dir);
int leaderboard_render(const char *submissions_dir, bool json_mode, const char *taskset_filter);

#endif /* SPACETIME_REPORT_H */
