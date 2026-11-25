#include "taskset.hpp"

#include <algorithm>
#include <array>
#include <format>

#include "sha256.hpp"

namespace spacetime {

namespace {

constexpr std::array<const char*, 4> kTasksetFiles = {"prompt.txt", "setup.sh", "test.sh", "meta.sh"};

std::string file_digest(const std::string& dir, const char* file) {
    return sha256_file(dir + "/" + file).value_or("-");
}

} // namespace

std::string taskset_manifest(const std::vector<BenchmarkTask>& tasks) {
    std::vector<std::string> rows;
    rows.reserve(tasks.size());

    for (const BenchmarkTask& task : tasks) {
        const std::size_t slash = task.task_dir.rfind('/');
        std::string row = slash == std::string::npos ? task.task_dir : task.task_dir.substr(slash + 1);
        if (row.empty()) row = task.id;

        for (const char* file : kTasksetFiles) {
            row += ':';
            row += file_digest(task.task_dir, file);
        }
        rows.push_back(std::move(row));
    }

    std::sort(rows.begin(), rows.end());

    std::string manifest;
    for (const std::string& row : rows) {
        manifest += row;
        manifest += '\n';
    }
    return manifest;
}

TasksetIdentity taskset_compute(const std::vector<BenchmarkTask>& tasks) {
    TasksetIdentity identity;
    identity.size = tasks.size();
    identity.hash = sha256_hex(taskset_manifest(tasks));
    identity.id = std::format("{}-{}-{}", kTasksetScheme, tasks.size(), identity.hash.substr(0, 8));
    return identity;
}

} // namespace spacetime
