#define _GNU_SOURCE
#include <getopt.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "agent.h"
#include "docker.h"
#include "eval.h"
#include "report.h"
#include "task.h"
#include "taskset.h"
#include "tui.h"
#include "types.h"

static void handle_sigint(int sig) {
    (void)sig;
    fprintf(stderr, "\n\033[33mReceived SIGINT / Ctrl+C. Cleaning up active sandbox containers...\033[0m\n");
    docker_cleanup_all_active_containers();
    _exit(130);
}

static void print_usage(const char *prog) {
    tui_print_banner();
    printf("USAGE:\n");
    printf("    %s [SUBCOMMAND] [OPTIONS]\n\n", prog);
    printf("SUBCOMMANDS:\n");
    printf("    run <task-id>       Execute benchmark evaluation on a single task\n");
    printf("    eval-all            Run all benchmark tasks and produce evaluation report\n");
    printf("    list                List all available benchmark tasks\n");
    printf("    info <task-id>      Show detailed information and objective for a task\n");
    printf("    clean               Remove any dangling or active spacetime sandbox containers\n");
    printf("    build-image         Build the sandbox Docker image\n");
    printf("    submit              Turn a benchmark report into a leaderboard submission\n");
    printf("    leaderboard         View ranked leaderboard results\n");
    printf("    help                Print this help menu\n\n");
    printf("OPTIONS:\n");
    printf("    -a, --agent <name>      Harness name (claude, gemini, agy, codex, aider)\n");
    printf("        --agent-cmd <cmd>   Custom in-container agent command string\n");
    printf("    -i, --image <image>     Docker sandbox image (default: %s)\n", DEFAULT_SANDBOX_IMAGE);
    printf("    -t, --tasks-dir <path>  Path to tasks directory (default: tasks)\n");
    printf("        --timeout <secs>    Override timeout per task in seconds\n");
    printf("        --force-rebuild     Force rebuild of Docker sandbox image\n\n");
}

int main(int argc, char **argv) {
    signal(SIGINT, handle_sigint);
    signal(SIGTERM, handle_sigint);

    char agent_name[128] = "claude";
    char agent_cmd[512] = "";
    char image[128] = DEFAULT_SANDBOX_IMAGE;
    char tasks_dir[512] = "tasks";
    uint64_t timeout_override = 0;
    bool force_rebuild = false;

    static struct option long_options[] = {
        {"agent", required_argument, 0, 'a'},
        {"agent-cmd", required_argument, 0, 1001},
        {"image", required_argument, 0, 'i'},
        {"tasks-dir", required_argument, 0, 't'},
        {"timeout", required_argument, 0, 1002},
        {"force-rebuild", no_argument, 0, 1003},
        {"help", no_argument, 0, 'h'},
        {0, 0, 0, 0}
    };

    int opt;
    int opt_idx = 0;
    while ((opt = getopt_long(argc, argv, "+a:i:t:h", long_options, &opt_idx)) != -1) {
        switch (opt) {
            case 'a':
                strncpy(agent_name, optarg, sizeof(agent_name) - 1);
                break;
            case 1001:
                strncpy(agent_cmd, optarg, sizeof(agent_cmd) - 1);
                break;
            case 'i':
                strncpy(image, optarg, sizeof(image) - 1);
                break;
            case 't':
                strncpy(tasks_dir, optarg, sizeof(tasks_dir) - 1);
                break;
            case 1002:
                timeout_override = strtoull(optarg, NULL, 10);
                break;
            case 1003:
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

    const char *subcmd = argv[optind++];

    AgentProfile agent;
    agent_profile_from_name_or_cmd(agent_name, agent_cmd[0] ? agent_cmd : NULL, &agent);

    if (strcmp(subcmd, "list") == 0) {
        BenchmarkTask *tasks = NULL;
        size_t count = 0;
        if (load_all_tasks(tasks_dir, &tasks, &count) != 0 || count == 0) {
            fprintf(stderr, "Failed to load tasks from %s\n", tasks_dir);
            return 1;
        }
        tui_print_task_list(tasks, count);
        free_tasks(tasks, count);
        return 0;
    }

    if (strcmp(subcmd, "info") == 0) {
        if (optind >= argc) {
            fprintf(stderr, "Error: Missing task ID for 'info'\n");
            return 1;
        }
        const char *task_id = argv[optind];
        BenchmarkTask task;
        if (find_task_by_id(tasks_dir, task_id, &task) != 0) {
            fprintf(stderr, "Task '%s' not found in %s\n", task_id, tasks_dir);
            return 1;
        }
        tui_print_task_info(&task);
        if (task.prompt) free(task.prompt);
        return 0;
    }

    if (strcmp(subcmd, "clean") == 0) {
        printf("Cleaning up all active spacetime Docker sandbox containers...\n");
        docker_cleanup_all_active_containers();
        printf("Done.\n");
        return 0;
    }

    if (strcmp(subcmd, "build-image") == 0) {
        if (docker_ensure_image(image, force_rebuild ? true : true) != 0) {
            return 1;
        }
        return 0;
    }

    if (strcmp(subcmd, "run") == 0) {
        if (optind >= argc) {
            fprintf(stderr, "Error: Missing task ID for 'run'\n");
            return 1;
        }
        const char *task_id = argv[optind];
        BenchmarkTask task;
        if (find_task_by_id(tasks_dir, task_id, &task) != 0) {
            fprintf(stderr, "Task '%s' not found in %s\n", task_id, tasks_dir);
            return 1;
        }

        if (docker_check_daemon() != 0) {
            fprintf(stderr, "\033[31mError: Cannot connect to Docker daemon at %s. Is Docker running?\033[0m\n", DOCKER_SOCK_PATH);
            return 1;
        }

        TaskResult res;
        eval_run_task(&task, &agent, image, timeout_override, false, &res);
        int exit_code = res.passed ? 0 : 1;
        free_task_result(&res);
        if (task.prompt) free(task.prompt);
        return exit_code;
    }

    if (strcmp(subcmd, "eval-all") == 0) {
        char *output_file = NULL;
        for (int i = optind; i < argc; i++) {
            if (strcmp(argv[i], "-o") == 0 || strcmp(argv[i], "--output") == 0) {
                if (i + 1 < argc) output_file = argv[++i];
            }
        }

        if (docker_check_daemon() != 0) {
            fprintf(stderr, "\033[31mError: Cannot connect to Docker daemon at %s. Is Docker running?\033[0m\n", DOCKER_SOCK_PATH);
            return 1;
        }

        BenchmarkTask *tasks = NULL;
        size_t count = 0;
        if (load_all_tasks(tasks_dir, &tasks, &count) != 0 || count == 0) {
            fprintf(stderr, "Failed to load tasks from %s\n", tasks_dir);
            return 1;
        }

        BenchmarkSuiteResult suite;
        eval_run_suite(tasks, count, &agent, image, timeout_override, output_file, &suite);
        free_tasks(tasks, count);
        free_benchmark_suite_result(&suite);
        return 0;
    }

    if (strcmp(subcmd, "submit") == 0) {
        char *report_path = NULL;
        char *user = NULL;
        char *tier = "community";
        char *source = NULL;
        char *notes = NULL;
        char *out_dir = DEFAULT_SUBMISSIONS_DIR;

        for (int i = optind; i < argc; i++) {
            if ((strcmp(argv[i], "-r") == 0 || strcmp(argv[i], "--report") == 0) && i + 1 < argc) {
                report_path = argv[++i];
            } else if (strcmp(argv[i], "--user") == 0 && i + 1 < argc) {
                user = argv[++i];
            } else if (strcmp(argv[i], "--tier") == 0 && i + 1 < argc) {
                tier = argv[++i];
            } else if (strcmp(argv[i], "--source") == 0 && i + 1 < argc) {
                source = argv[++i];
            } else if (strcmp(argv[i], "--notes") == 0 && i + 1 < argc) {
                notes = argv[++i];
            } else if (strcmp(argv[i], "--out-dir") == 0 && i + 1 < argc) {
                out_dir = argv[++i];
            }
        }

        if (!report_path) {
            fprintf(stderr, "Error: --report <path> is required for submit\n");
            return 1;
        }

        return submit_create(report_path, user, tier, source, notes, out_dir);
    }

    if (strcmp(subcmd, "leaderboard") == 0) {
        char *dir = DEFAULT_SUBMISSIONS_DIR;
        bool json_mode = false;
        char *taskset_filter = NULL;

        for (int i = optind; i < argc; i++) {
            if (strcmp(argv[i], "--dir") == 0 && i + 1 < argc) {
                dir = argv[++i];
            } else if (strcmp(argv[i], "--json") == 0) {
                json_mode = true;
            } else if (strcmp(argv[i], "--taskset") == 0 && i + 1 < argc) {
                taskset_filter = argv[++i];
            }
        }

        return leaderboard_render(dir, json_mode, taskset_filter);
    }

    if (strcmp(subcmd, "help") == 0 || strcmp(subcmd, "--help") == 0) {
        print_usage(argv[0]);
        return 0;
    }

    fprintf(stderr, "Unknown subcommand: '%s'\nRun '%s help' for usage.\n", subcmd, argv[0]);
    return 1;
}
