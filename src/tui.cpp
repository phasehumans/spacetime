#include "tui.hpp"

#include <algorithm>
#include <initializer_list>
#include <string>
#include <string_view>

#include "util.hpp"

namespace spacetime {

namespace {

bool contains_any(std::string_view id, std::initializer_list<std::string_view> needles) {
    for (std::string_view needle : needles) {
        if (id.find(needle) != std::string_view::npos) return true;
    }
    return false;
}

const char* get_category_tag(std::string_view id) {
    if (contains_any(id, {"nginx", "service", "process", "permission"})) return "sysadmin";
    if (contains_any(id, {"port", "dns", "ssh", "curl", "network"})) return "network";
    if (contains_any(id, {"git"})) return "git";
    if (contains_any(id, {"docker"})) return "docker";
    if (contains_any(id, {"sqlite", "sql", "postgres"})) return "database";
    if (contains_any(id, {"json", "regex", "log", "csv", "tar"})) return "data";
    return "general";
}

// First line of the prompt, capped at 40 bytes.
std::string prompt_preview(const std::string& prompt) {
    if (prompt.empty()) return "-";
    std::string preview = prompt.substr(0, 40);
    preview.resize(std::min(preview.find('\n'), preview.size()));
    if (prompt.size() > 40) preview += "...";
    return preview;
}

} // namespace

void tui_print_banner() {
    print("\n");
    print("\033[1;36m  ███████╗██████╗  █████╗  ██████╗███████╗████████╗██╗███╗   ███╗███████╗\033[0m\n");
    print("\033[1;36m  ██╔════╝██╔══██╗██╔══██╗██╔════╝██╔════╝╚══██╔══╝██║████╗ ████║██╔════╝\033[0m\n");
    print("\033[1;36m  ███████╗██████╔╝███████║██║     █████╗     ██║   ██║██╔████╔██║█████╗  \033[0m\n");
    print("\033[1;36m  ╚════██║██╔═══╝ ██╔══██║██║     ██╔══╝     ██║   ██║██║╚██╔╝██║██╔══╝  \033[0m\n");
    print("\033[1;36m  ███████║██║     ██║  ██║╚██████╗███████╗   ██║   ██║██║ ╚═╝ ██║███████╗\033[0m\n");
    print("\033[1;36m  ╚══════╝╚═╝     ╚═╝  ╚═╝ ╚═════╝╚══════╝   ╚═╝   ╚═╝╚═╝     ╚═╝╚══════╝\033[0m\n");
    print("\033[2m        in-container benchmark arena for autonomous coding agents (C++20)\033[0m\n\n");
}

void tui_print_task_list(const std::vector<BenchmarkTask>& tasks) {
    tui_print_banner();
    print("\033[1mBENCHMARK TASKS ({} Total)\033[0m\n\n", tasks.size());
    print("  \033[2m{:<30} {:<12} {:<10} {}\033[0m\n", "TASK ID", "CATEGORY", "TIMEOUT", "OBJECTIVE");
    print("  ----------------------------------------------------------------------------------------------------\n");
    for (const BenchmarkTask& t : tasks) {
        print("  \033[1m{:<30}\033[0m \033[36m[{:<8}]\033[0m \033[2m{:>4}s\033[0m      {}\n", t.id,
              get_category_tag(t.id), t.timeout_secs, prompt_preview(t.prompt));
    }
    print("\n");
}

void tui_print_task_info(const BenchmarkTask& task) {
    print("\n\033[1;36m• TASK SPECIFICATION: {} •\033[0m\n", task.id);
    print("  \033[2mName:        \033[0m{}\n", task.name);
    print("  \033[2mDescription: \033[0m{}\n", task.description.empty() ? "-" : task.description);
    print("  \033[2mBase Image:  \033[0m{}\n", task.base_image);
    print("  \033[2mMax Turns:   \033[0m{}\n", task.max_turns);
    print("  \033[2mTimeout:     \033[0m{}s\n", task.timeout_secs);
    print("  \033[2mTask Dir:    \033[0m{}\n", task.task_dir);
    print("\n\033[1m• Objective Prompt:\033[0m\n");
    print("  {}\n\n", task.prompt);
}

} // namespace spacetime
