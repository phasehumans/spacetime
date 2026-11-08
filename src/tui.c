#include "tui.h"
#include <stdio.h>
#include <string.h>

void tui_print_banner(void) {
    printf("\n");
    printf("\033[1;36m  ███████╗██████╗  █████╗  ██████╗███████╗████████╗██╗███╗   ███╗███████╗\033[0m\n");
    printf("\033[1;36m  ██╔════╝██╔══██╗██╔══██╗██╔════╝██╔════╝╚══██╔══╝██║████╗ ████║██╔════╝\033[0m\n");
    printf("\033[1;36m  ███████╗██████╔╝███████║██║     █████╗     ██║   ██║██╔████╔██║█████╗  \033[0m\n");
    printf("\033[1;36m  ╚════██║██╔═══╝ ██╔══██║██║     ██╔══╝     ██║   ██║██║╚██╔╝██║██╔══╝  \033[0m\n");
    printf("\033[1;36m  ███████║██║     ██║  ██║╚██████╗███████╗   ██║   ██║██║ ╚═╝ ██║███████╗\033[0m\n");
    printf("\033[1;36m  ╚══════╝╚═╝     ╚═╝  ╚═╝ ╚═════╝╚══════╝   ╚═╝   ╚═╝╚═╝     ╚═╝╚══════╝\033[0m\n");
    printf("\033[2m        in-container benchmark arena for autonomous coding agents (C11)\033[0m\n\n");
}

static const char *get_category_tag(const char *id) {
    if (strstr(id, "nginx") || strstr(id, "service") || strstr(id, "process") || strstr(id, "permission")) return "sysadmin";
    if (strstr(id, "port") || strstr(id, "dns") || strstr(id, "ssh") || strstr(id, "curl") || strstr(id, "network")) return "network";
    if (strstr(id, "git")) return "git";
    if (strstr(id, "docker")) return "docker";
    if (strstr(id, "sqlite") || strstr(id, "sql") || strstr(id, "postgres")) return "database";
    if (strstr(id, "json") || strstr(id, "regex") || strstr(id, "log") || strstr(id, "csv") || strstr(id, "tar")) return "data";
    return "general";
}

void tui_print_task_list(const BenchmarkTask *tasks, size_t count) {
    tui_print_banner();
    printf("\033[1mBENCHMARK TASKS (%zu Total)\033[0m\n\n", count);
    printf("  \033[2m%-30s %-12s %-10s %s\033[0m\n", "TASK ID", "CATEGORY", "TIMEOUT", "OBJECTIVE");
    printf("  ----------------------------------------------------------------------------------------------------\n");
    for (size_t i = 0; i < count; i++) {
        const BenchmarkTask *t = &tasks[i];
        const char *cat = get_category_tag(t->id);
        char preview[45];
        if (t->prompt && *t->prompt) {
            strncpy(preview, t->prompt, 40);
            preview[40] = '\0';
            char *nl = strchr(preview, '\n');
            if (nl) *nl = '\0';
            if (strlen(t->prompt) > 40) strcat(preview, "...");
        } else {
            strcpy(preview, "-");
        }
        printf("  \033[1m%-30s\033[0m \033[36m[%-8s]\033[0m \033[2m%4llus\033[0m      %s\n",
            t->id, cat, (unsigned long long)t->timeout_secs, preview);
    }
    printf("\n");
}

void tui_print_task_info(const BenchmarkTask *task) {
    printf("\n\033[1;36m• TASK SPECIFICATION: %s •\033[0m\n", task->id);
    printf("  \033[2mName:        \033[0m%s\n", task->name);
    printf("  \033[2mDescription: \033[0m%s\n", task->description[0] ? task->description : "-");
    printf("  \033[2mBase Image:  \033[0m%s\n", task->base_image);
    printf("  \033[2mMax Turns:   \033[0m%u\n", task->max_turns);
    printf("  \033[2mTimeout:     \033[0m%llus\n", (unsigned long long)task->timeout_secs);
    printf("  \033[2mTask Dir:    \033[0m%s\n", task->task_dir);
    printf("\n\033[1m• Objective Prompt:\033[0m\n");
    printf("  %s\n\n", task->prompt ? task->prompt : "");
}
