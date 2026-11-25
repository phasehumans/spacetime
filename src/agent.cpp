#include "agent.hpp"

#include <array>
#include <cstdlib>
#include <format>

#include "util.hpp"

namespace spacetime {

namespace {

constexpr std::array<const char*, 7> kPassthroughKeys = {
    "ANTHROPIC_API_KEY", "OPENAI_API_KEY", "GEMINI_API_KEY", "GOOGLE_API_KEY",
    "GROQ_API_KEY",      "DEEPSEEK_API_KEY", "OPENROUTER_API_KEY",
};

// `<base> [--model "<model>"] <tail>`, skipping the model flag when unset.
std::string with_model(std::string_view base, const std::string& model, std::string_view tail) {
    std::string cmd(base);
    if (!model.empty()) cmd += std::format(" --model \"{}\"", model);
    cmd += tail;
    return cmd;
}

} // namespace

AgentProfile agent_profile_from_name_or_cmd(std::string_view name_or_harness, std::string_view custom_cmd) {
    AgentProfile profile;

    if (!custom_cmd.empty()) {
        profile.harness = HarnessType::Custom;
        profile.name = "Custom CLI";
        profile.custom_cmd = custom_cmd;
        return profile;
    }

    if (name_or_harness.empty()) name_or_harness = "claude";

    if (iequals(name_or_harness, "claude") || iequals(name_or_harness, "claude-code")) {
        profile.harness = HarnessType::ClaudeCode;
        profile.name = "Claude Code";
        profile.model = "claude-3-7-sonnet-20250219";
        profile.prompt_cost_per_m = 3.00;
        profile.completion_cost_per_m = 15.00;
    } else if (iequals(name_or_harness, "gemini") || iequals(name_or_harness, "gemini-cli")) {
        profile.harness = HarnessType::GeminiCli;
        profile.name = "Gemini CLI";
        profile.model = "gemini-2.0-flash";
        profile.prompt_cost_per_m = 0.10;
        profile.completion_cost_per_m = 0.40;
    } else if (iequals(name_or_harness, "agy") || iequals(name_or_harness, "antigravity")) {
        profile.harness = HarnessType::Antigravity;
        profile.name = "Antigravity";
        profile.model = "gemini-2.0-flash";
        profile.prompt_cost_per_m = 0.10;
        profile.completion_cost_per_m = 0.40;
    } else if (iequals(name_or_harness, "codex")) {
        profile.harness = HarnessType::Codex;
        profile.name = "Codex";
        profile.model = "gpt-4o";
        profile.prompt_cost_per_m = 2.50;
        profile.completion_cost_per_m = 10.00;
    } else if (iequals(name_or_harness, "aider")) {
        profile.harness = HarnessType::Aider;
        profile.name = "Aider";
        profile.model = "claude-3-7-sonnet";
        profile.prompt_cost_per_m = 3.00;
        profile.completion_cost_per_m = 15.00;
    } else {
        profile.harness = HarnessType::Custom;
        profile.name = name_or_harness;
        profile.custom_cmd = name_or_harness;
    }

    return profile;
}

std::string agent_profile_build_cmd(const AgentProfile& profile) {
    switch (profile.harness) {
        case HarnessType::ClaudeCode:
            return with_model("claude -p \"$SPACETIME_PROMPT\"", profile.model, " --dangerously-skip-permissions");
        case HarnessType::GeminiCli:
            return with_model("gemini --prompt \"$SPACETIME_PROMPT\"", profile.model, " --yes --headless");
        case HarnessType::Antigravity:
            return with_model("agy --prompt \"$SPACETIME_PROMPT\"", profile.model, " --yes");
        case HarnessType::Codex:
            return with_model("codex run --prompt \"$SPACETIME_PROMPT\"", profile.model, "");
        case HarnessType::Aider:
            return "aider --message \"$SPACETIME_PROMPT\" --yes --no-git";
        default:
            return profile.custom_cmd.empty() ? "bash" : profile.custom_cmd;
    }
}

std::vector<std::string> agent_profile_get_env(std::string_view prompt) {
    std::vector<std::string> env;
    env.push_back(std::format("SPACETIME_PROMPT={}", prompt));
    env.push_back(std::format("PROMPT={}", prompt));

    for (const char* key : kPassthroughKeys) {
        if (const char* value = std::getenv(key)) {
            env.push_back(std::format("{}={}", key, value));
        }
    }
    return env;
}

} // namespace spacetime
