#pragma once

#include <string>
#include <vector>

#include "types.hpp"

namespace spacetime {

inline constexpr char kTasksetScheme[] = "v1";

// One `folder:prompt:setup:test:meta` row of SHA-256 digests per task, sorted
// and newline-terminated. A missing file is recorded as "-".
std::string taskset_manifest(const std::vector<BenchmarkTask>& tasks);

// Content-addressed identity of a task set: the SHA-256 of its manifest.
TasksetIdentity taskset_compute(const std::vector<BenchmarkTask>& tasks);

} // namespace spacetime
