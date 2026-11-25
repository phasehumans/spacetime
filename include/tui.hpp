#pragma once

#include <vector>

#include "types.hpp"

namespace spacetime {

void tui_print_banner();
void tui_print_task_list(const std::vector<BenchmarkTask>& tasks);
void tui_print_task_info(const BenchmarkTask& task);

} // namespace spacetime
