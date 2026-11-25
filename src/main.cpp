#include <csignal>
#include <cstdint>
#include <cstdlib>
#include <exception>
#include <string>
#include <string_view>

#include <getopt.h>
#include <unistd.h>

#include "agent.hpp"
#include "docker.hpp"
#include "eval.hpp"
#include "task.hpp"
#include "tui.hpp"
#include "types.hpp"
#include "util.hpp"

using namespace spacetime;

namespace {

void handle_signal(int) {
    static constexpr char kMessage[] =
        "\n\033[33mReceived SIGINT / Ctrl+C. Cleaning up active sandbox containers...\033[0m\n";
    [[maybe_unused]] const ssize_t written = ::write(STDERR_FILENO, kMessage, sizeof(kMessage) - 1);
    docker::cleanup_all_active_containers();
    ::_exit(130);
}

void print_usage(const char* prog) {
    tui_print_banner();
    print("USAGE:\n");
    print("    {} [SUBCOMMAND] [OPTIONS]\n\n", prog);
    print("SUBCOMMANDS:\n");
    print("    run <task-id>       Execute benchmark evaluation on a single task\n");
    print("    eval-all            Run all benchmark tasks and produce evaluation report\n");
    print("    list                List all available benchmark tasks\n");
    print("    info <task-id>      Show detailed information and objective for a task\n");
    print("    clean               Remove any dangling or active spacetime sandbox containers\n");
    print("    build-image         Build the sandbox Docker image\n");
    print("    help                Print this help menu\n\n");
    print("OPTIONS:\n");
    print("    -a, --agent <name>      Harness name (claude, gemini, agy, codex, aider)\n");
    print("        --agent-cmd <cmd>   Custom in-container agent command string\n");
    print("    -i, --image <image>     Docker sandbox image (default: {})\n", kDefaultSandboxImage);
    print("    -t, --tasks-dir <path>  Path to tasks directory (default: tasks)\n");
    print("    -o, --output <file>     Report path for eval-all (default: results/spacetime_results_<time>.json)\n");
    print("        --timeout <secs>    Override timeout per task in seconds\n");
    print("        --force-rebuild     Force rebuild of Docker sandbox image\n\n");
}

bool require_docker(const std::string& image, bool force_rebuild) {
    if (!docker::check_daemon()) {
        eprint("\033[31mError: Cannot connect to Docker daemon at {}. Is Docker running?\033[0m\n",
               docker::kSockPath);
        return false;
    }
    return docker::ensure_image(image, force_rebuild);
}

int run(int argc, char** argv) {
    std::signal(SIGINT, handle_signal);
    std::signal(SIGTERM, handle_signal);

    std::string agent_name = "claude";
    std::string agent_cmd;
    std::string image = kDefaultSandboxImage;
    std::string tasks_dir = "tasks";
    std::string output_file;
    std::uint64_t timeout_override = 0;
    bool force_rebuild = false;

    enum : int { kOptAgentCmd = 1001, kOptTimeout, kOptForceRebuild };
    static const option long_options[] = {
        {"agent", required_argument, nullptr, 'a'},
        {"agent-cmd", required_argument, nullptr, kOptAgentCmd},
        {"image", required_argument, nullptr, 'i'},
        {"tasks-dir", required_argument, nullptr, 't'},
        {"output", required_argument, nullptr, 'o'},
        {"timeout", required_argument, nullptr, kOptTimeout},
        {"force-rebuild", no_argument, nullptr, kOptForceRebuild},
        {"help", no_argument, nullptr, 'h'},
        {nullptr, 0, nullptr, 0},
    };

    int opt;
    while ((opt = getopt_long(argc, argv, "a:i:t:o:h", long_options, nullptr)) != -1) {
        switch (opt) {
            case 'a':
                agent_name = optarg;
                break;
            case kOptAgentCmd:
                agent_cmd = optarg;
                break;
            case 'i':
                image = optarg;
                break;
            case 't':
                tasks_dir = optarg;
                break;
            case 'o':
                output_file = optarg;
                break;
            case kOptTimeout:
                timeout_override = std::strtoull(optarg, nullptr, 10);
                break;
            case kOptForceRebuild:
                force_rebuild = true;
                break;
            case 'h':
                print_usage(argv[0]);
                return 0;
            default:
                break;
        }
    }

    if (optind >= argc) {
        print_usage(argv[0]);
        return 0;
    }

    const std::string_view subcmd = argv[optind++];
    const AgentProfile agent = agent_profile_from_name_or_cmd(agent_name, agent_cmd);

    if (subcmd == "list") {
        const auto tasks = load_all_tasks(tasks_dir);
        if (!tasks || tasks->empty()) {
            eprint("Failed to load tasks from {}\n", tasks_dir);
            return 1;
        }
        tui_print_task_list(*tasks);
        return 0;
    }

    if (subcmd == "info") {
        if (optind >= argc) {
            eprint("Error: Missing task ID for 'info'\n");
            return 1;
        }
        const char* task_id = argv[optind];
        const auto task = find_task_by_id(tasks_dir, task_id);
        if (!task) {
            eprint("Task '{}' not found in {}\n", task_id, tasks_dir);
            return 1;
        }
        tui_print_task_info(*task);
        return 0;
    }

    if (subcmd == "clean") {
        print("Cleaning up all active spacetime Docker sandbox containers...\n");
        docker::cleanup_all_active_containers();
        print("Done.\n");
        return 0;
    }

    if (subcmd == "build-image") {
        return docker::ensure_image(image, true) ? 0 : 1;
    }

    if (subcmd == "run") {
        if (optind >= argc) {
            eprint("Error: Missing task ID for 'run'\n");
            return 1;
        }
        const char* task_id = argv[optind];
        const auto task = find_task_by_id(tasks_dir, task_id);
        if (!task) {
            eprint("Task '{}' not found in {}\n", task_id, tasks_dir);
            return 1;
        }

        if (!require_docker(image, force_rebuild)) return 1;

        const TaskResult res = eval_run_task(*task, agent, image, timeout_override, false);
        return res.passed ? 0 : 1;
    }

    if (subcmd == "eval-all") {
        // Also accept the output flag after a `--` separator.
        for (int i = optind; i < argc; ++i) {
            const std::string_view arg = argv[i];
            if ((arg == "-o" || arg == "--output") && i + 1 < argc) output_file = argv[++i];
        }

        if (!require_docker(image, force_rebuild)) return 1;

        const auto tasks = load_all_tasks(tasks_dir);
        if (!tasks || tasks->empty()) {
            eprint("Failed to load tasks from {}\n", tasks_dir);
            return 1;
        }

        eval_run_suite(*tasks, agent, image, timeout_override, output_file);
        return 0;
    }

    if (subcmd == "help" || subcmd == "--help") {
        print_usage(argv[0]);
        return 0;
    }

    eprint("Unknown subcommand: '{}'\nRun '{} help' for usage.\n", subcmd, argv[0]);
    return 1;
}

} // namespace

int main(int argc, char** argv) {
    try {
        return run(argc, argv);
    } catch (const std::exception& e) {
        eprint("\033[31mFatal error: {}\033[0m\n", e.what());
        docker::cleanup_all_active_containers();
        return 1;
    }
}
