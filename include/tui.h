#ifndef SPACETIME_TUI_H
#define SPACETIME_TUI_H

#include "types.h"
#include <stddef.h>

void tui_print_banner(void);
void tui_print_task_list(const BenchmarkTask *tasks, size_t count);
void tui_print_task_info(const BenchmarkTask *task);

#endif /* SPACETIME_TUI_H */
