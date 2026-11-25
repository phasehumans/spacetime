#include "task.hpp"

#include <algorithm>
#include <cstdlib>
#include <filesystem>
#include <system_error>

#include "util.hpp"

namespace spacetime {

namespace fs = std::filesystem;

namespace {

bool path_exists(const std::string& path) {
    std::error_code ec;
    return fs::exists(path, ec);
}

// Parses the `KEY="value"` lines of a meta.sh / meta.toml file into `task`.
void parse_meta(std::string_view content, BenchmarkTask& task) {
    std::size_t pos = 0;
    while (pos < content.size()) {
        std::size_t end = content.find_first_of("\r\n", pos);
        if (end == std::string_view::npos) end = content.size();
        const std::string_view line = trim(content.substr(pos, end - pos));
        pos = end + 1;

        if (line.empty() || line.front() == '#') continue;
        const std::size_t eq = line.find('=');
        if (eq == std::string_view::npos) continue;

        const std::string_view key = trim(line.substr(0, eq));
        std::string_view val = trim(line.substr(eq + 1));

        // Strip surrounding quotes.
        if (val.size() >= 2 && (val.front() == '"' || val.front() == '\'') && val.back() == val.front()) {
            val = val.substr(1, val.size() - 2);
        }
        const std::string value(val);

        if (iequals(key, "TASK_ID") || iequals(key, "ID")) {
            task.id = value;
        } else if (iequals(key, "TASK_NAME") || iequals(key, "NAME")) {
            task.name = value;
        } else if (iequals(key, "BASE_IMAGE") || iequals(key, "IMAGE")) {
            task.base_image = value;
        } else if (iequals(key, "MAX_TURNS")) {
            task.max_turns = static_cast<std::uint32_t>(std::strtoul(value.c_str(), nullptr, 10));
        } else if (iequals(key, "TIMEOUT_SECS") || iequals(key, "TIMEOUT")) {
            task.timeout_secs = std::strtoull(value.c_str(), nullptr, 10);
        } else if (iequals(key, "DESCRIPTION") || iequals(key, "DESC")) {
            task.description = value;
        }
    }
}

std::optional<BenchmarkTask> load_task_from_dir(const std::string& dir_path) {
    const std::size_t slash = dir_path.rfind('/');
    const std::string folder_name = slash == std::string::npos ? dir_path : dir_path.substr(slash + 1);

    BenchmarkTask task;
    task.id = folder_name;
    task.name = folder_name;
    task.task_dir = dir_path;
    task.setup_script = dir_path + "/setup.sh";
    task.test_script = dir_path + "/test.sh";

    if (auto prompt = read_file(dir_path + "/prompt.txt")) {
        task.prompt = std::string(trim(*prompt));
    }

    auto meta = read_file(dir_path + "/meta.sh");
    if (!meta) meta = read_file(dir_path + "/meta.toml");
    if (meta) parse_meta(*meta, task);

    if (!path_exists(task.setup_script) || !path_exists(task.test_script)) {
        return std::nullopt;
    }
    return task;
}

} // namespace

std::optional<std::vector<BenchmarkTask>> load_all_tasks(const std::string& tasks_dir) {
    std::error_code ec;
    fs::directory_iterator it(tasks_dir, ec);
    if (ec) return std::nullopt;

    std::vector<BenchmarkTask> tasks;
    for (const fs::directory_iterator end; it != end; it.increment(ec)) {
        if (ec) break;

        const std::string path = tasks_dir + "/" + it->path().filename().string();
        std::error_code stat_ec;
        if (!fs::is_directory(path, stat_ec)) continue;
        if (!path_exists(path + "/prompt.txt") && !path_exists(path + "/meta.sh")) continue;

        if (auto task = load_task_from_dir(path)) {
            tasks.push_back(std::move(*task));
        }
    }

    std::sort(tasks.begin(), tasks.end(),
              [](const BenchmarkTask& a, const BenchmarkTask& b) { return a.id < b.id; });
    return tasks;
}

std::optional<BenchmarkTask> find_task_by_id(const std::string& tasks_dir, std::string_view id_query) {
    auto tasks = load_all_tasks(tasks_dir);
    if (!tasks || tasks->empty()) return std::nullopt;

    // 1. Exact match
    auto found = std::find_if(tasks->begin(), tasks->end(),
                              [&](const BenchmarkTask& t) { return t.id == id_query; });

    // 2. Prefix or substring match
    if (found == tasks->end()) {
        std::string_view clean_query = id_query;
        if (clean_query.starts_with("task-")) clean_query.remove_prefix(5);

        found = std::find_if(tasks->begin(), tasks->end(), [&](const BenchmarkTask& t) {
            return t.id.find(id_query) != std::string::npos ||
                   t.id.find(clean_query) != std::string::npos ||
                   icontains(t.name, id_query);
        });
    }

    if (found == tasks->end()) return std::nullopt;
    return std::move(*found);
}

} // namespace spacetime
