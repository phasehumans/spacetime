#define _GNU_SOURCE
#include "agent.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <strings.h>

int agent_profile_from_name_or_cmd(const char *name_or_harness, const char *custom_cmd, AgentProfile *out_profile) {
    memset(out_profile, 0, sizeof(AgentProfile));
    out_profile->prompt_cost_per_m = 3.00;
    out_profile->completion_cost_per_m = 15.00;

    if (custom_cmd && *custom_cmd) {
        out_profile->harness = HARNESS_CUSTOM;
        strncpy(out_profile->name, "Custom CLI", sizeof(out_profile->name) - 1);
        strncpy(out_profile->custom_cmd, custom_cmd, sizeof(out_profile->custom_cmd) - 1);
        return 0;
    }

    if (!name_or_harness || !*name_or_harness) {
        name_or_harness = "claude";
    }

    if (strcasecmp(name_or_harness, "claude") == 0 || strcasecmp(name_or_harness, "claude-code") == 0) {
        out_profile->harness = HARNESS_CLAUDE_CODE;
        strncpy(out_profile->name, "Claude Code", sizeof(out_profile->name) - 1);
        strncpy(out_profile->model, "claude-3-7-sonnet-20250219", sizeof(out_profile->model) - 1);
        out_profile->prompt_cost_per_m = 3.00;
        out_profile->completion_cost_per_m = 15.00;
    } else if (strcasecmp(name_or_harness, "gemini") == 0 || strcasecmp(name_or_harness, "gemini-cli") == 0) {
        out_profile->harness = HARNESS_GEMINI_CLI;
        strncpy(out_profile->name, "Gemini CLI", sizeof(out_profile->name) - 1);
        strncpy(out_profile->model, "gemini-2.0-flash", sizeof(out_profile->model) - 1);
        out_profile->prompt_cost_per_m = 0.10;
        out_profile->completion_cost_per_m = 0.40;
    } else if (strcasecmp(name_or_harness, "agy") == 0 || strcasecmp(name_or_harness, "antigravity") == 0) {
        out_profile->harness = HARNESS_ANTIGRAVITY;
        strncpy(out_profile->name, "Antigravity", sizeof(out_profile->name) - 1);
        strncpy(out_profile->model, "gemini-2.0-flash", sizeof(out_profile->model) - 1);
        out_profile->prompt_cost_per_m = 0.10;
        out_profile->completion_cost_per_m = 0.40;
    } else if (strcasecmp(name_or_harness, "codex") == 0) {
        out_profile->harness = HARNESS_CODEX;
        strncpy(out_profile->name, "Codex", sizeof(out_profile->name) - 1);
        strncpy(out_profile->model, "gpt-4o", sizeof(out_profile->model) - 1);
        out_profile->prompt_cost_per_m = 2.50;
        out_profile->completion_cost_per_m = 10.00;
    } else if (strcasecmp(name_or_harness, "aider") == 0) {
        out_profile->harness = HARNESS_AIDER;
        strncpy(out_profile->name, "Aider", sizeof(out_profile->name) - 1);
        strncpy(out_profile->model, "claude-3-7-sonnet", sizeof(out_profile->model) - 1);
        out_profile->prompt_cost_per_m = 3.00;
        out_profile->completion_cost_per_m = 15.00;
    } else {
        out_profile->harness = HARNESS_CUSTOM;
        strncpy(out_profile->name, name_or_harness, sizeof(out_profile->name) - 1);
        strncpy(out_profile->custom_cmd, name_or_harness, sizeof(out_profile->custom_cmd) - 1);
    }

    return 0;
}

int agent_profile_build_cmd(const AgentProfile *profile, const char *prompt, char *out_cmd, size_t out_cmd_sz) {
    (void)prompt;
    switch (profile->harness) {
        case HARNESS_CLAUDE_CODE:
            if (profile->model[0]) {
                snprintf(out_cmd, out_cmd_sz, "claude -p \"$SPACETIME_PROMPT\" --model \"%s\" --dangerously-skip-permissions", profile->model);
            } else {
                snprintf(out_cmd, out_cmd_sz, "claude -p \"$SPACETIME_PROMPT\" --dangerously-skip-permissions");
            }
            break;
        case HARNESS_GEMINI_CLI:
            if (profile->model[0]) {
                snprintf(out_cmd, out_cmd_sz, "gemini --prompt \"$SPACETIME_PROMPT\" --model \"%s\" --yes --headless", profile->model);
            } else {
                snprintf(out_cmd, out_cmd_sz, "gemini --prompt \"$SPACETIME_PROMPT\" --yes --headless");
            }
            break;
        case HARNESS_ANTIGRAVITY:
            if (profile->model[0]) {
                snprintf(out_cmd, out_cmd_sz, "agy --prompt \"$SPACETIME_PROMPT\" --model \"%s\" --yes", profile->model);
            } else {
                snprintf(out_cmd, out_cmd_sz, "agy --prompt \"$SPACETIME_PROMPT\" --yes");
            }
            break;
        case HARNESS_CODEX:
            if (profile->model[0]) {
                snprintf(out_cmd, out_cmd_sz, "codex run --prompt \"$SPACETIME_PROMPT\" --model \"%s\"", profile->model);
            } else {
                snprintf(out_cmd, out_cmd_sz, "codex run --prompt \"$SPACETIME_PROMPT\"");
            }
            break;
        case HARNESS_AIDER:
            snprintf(out_cmd, out_cmd_sz, "aider --message \"$SPACETIME_PROMPT\" --yes --no-git");
            break;
        case HARNESS_CUSTOM:
        default:
            snprintf(out_cmd, out_cmd_sz, "%s", profile->custom_cmd[0] ? profile->custom_cmd : "bash");
            break;
    }
    return 0;
}

int agent_profile_get_env(const AgentProfile *profile, const char *prompt, char ***out_env, size_t *out_count) {
    (void)profile;
    size_t cap = 32;
    size_t count = 0;
    char **env = malloc(sizeof(char *) * cap);
    if (!env) return -1;

    char p1[4096];
    snprintf(p1, sizeof(p1), "SPACETIME_PROMPT=%s", prompt ? prompt : "");
    env[count++] = strdup(p1);

    char p2[4096];
    snprintf(p2, sizeof(p2), "PROMPT=%s", prompt ? prompt : "");
    env[count++] = strdup(p2);

    const char *pass_keys[] = {
        "ANTHROPIC_API_KEY", "OPENAI_API_KEY", "GEMINI_API_KEY", "GOOGLE_API_KEY",
        "GROQ_API_KEY", "DEEPSEEK_API_KEY", "OPENROUTER_API_KEY"
    };

    for (size_t i = 0; i < sizeof(pass_keys)/sizeof(pass_keys[0]); i++) {
        const char *val = getenv(pass_keys[i]);
        if (val) {
            char entry[4096];
            snprintf(entry, sizeof(entry), "%s=%s", pass_keys[i], val);
            env[count++] = strdup(entry);
        }
    }

    *out_env = env;
    *out_count = count;
    return 0;
}

void agent_profile_free_env(char **env, size_t count) {
    if (!env) return;
    for (size_t i = 0; i < count; i++) {
        if (env[i]) free(env[i]);
    }
    free(env);
}
