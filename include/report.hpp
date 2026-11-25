#pragma once

#include <string>

#include "types.hpp"

namespace spacetime {

// Serializes a suite result as pretty-printed JSON.
std::string report_to_json(const BenchmarkSuiteResult& suite);

// Writes the JSON report to `filepath`. Returns false if the file cannot be
// written.
bool report_save_json(const BenchmarkSuiteResult& suite, const std::string& filepath);

} // namespace spacetime
