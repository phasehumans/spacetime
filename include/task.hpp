#pragma once

#include <optional>
#include <string>
#include <string_view>
#include <vector>

#include "types.hpp"

namespace spacetime {

// Loads every task under `tasks_dir`, sorted by task id. Returns nullopt if
// the directory cannot be read.
std::optional<std::vector<BenchmarkTask>> load_all_tasks(const std::string& tasks_dir);

// Finds a task by exact id, falling back to a substring match on the id (with
// an optional "task-" prefix stripped) or a case-insensitive match on the name.
std::optional<BenchmarkTask> find_task_by_id(const std::string& tasks_dir, std::string_view id_query);

} // namespace spacetime
