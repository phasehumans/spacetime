#include "catch2/catch_amalgamated.hpp"

#include <algorithm>
#include <string>
#include <vector>

#include "agent.hpp"
#include "test_helpers.hpp"

using namespace spacetime;

namespace {

bool contains(const std::vector<std::string>& env, const std::string& entry) {
    return std::find(env.begin(), env.end(), entry) != env.end();
}

} // namespace

TEST_CASE("built-in harness names resolve to profiles", "[agent]") {
    const AgentProfile claude = agent_profile_from_name_or_cmd("claude");
    CHECK(claude.harness == HarnessType::ClaudeCode);
    CHECK(claude.name == "Claude Code");
    CHECK(claude.model == "claude-3-7-sonnet-20250219");
    CHECK(claude.prompt_cost_per_m == 3.00);
    CHECK(claude.completion_cost_per_m == 15.00);

    CHECK(agent_profile_from_name_or_cmd("Claude-Code").harness == HarnessType::ClaudeCode);
    CHECK(agent_profile_from_name_or_cmd("GEMINI").harness == HarnessType::GeminiCli);
    CHECK(agent_profile_from_name_or_cmd("gemini-cli").name == "Gemini CLI");
    CHECK(agent_profile_from_name_or_cmd("agy").harness == HarnessType::Antigravity);
    CHECK(agent_profile_from_name_or_cmd("antigravity").name == "Antigravity");
    CHECK(agent_profile_from_name_or_cmd("codex").model == "gpt-4o");
    CHECK(agent_profile_from_name_or_cmd("aider").harness == HarnessType::Aider);
}

TEST_CASE("an empty name defaults to claude", "[agent]") {
    CHECK(agent_profile_from_name_or_cmd("").harness == HarnessType::ClaudeCode);
}

TEST_CASE("custom commands and unknown names become custom profiles", "[agent]") {
    const AgentProfile custom = agent_profile_from_name_or_cmd("claude", "my-agent --flag");
    CHECK(custom.harness == HarnessType::Custom);
    CHECK(custom.name == "Custom CLI");
    CHECK(custom.custom_cmd == "my-agent --flag");
    CHECK(custom.model.empty());
    CHECK(agent_profile_build_cmd(custom) == "my-agent --flag");

    const AgentProfile unknown = agent_profile_from_name_or_cmd("some-tool run");
    CHECK(unknown.harness == HarnessType::Custom);
    CHECK(unknown.name == "some-tool run");
    CHECK(agent_profile_build_cmd(unknown) == "some-tool run");

    CHECK(agent_profile_build_cmd(AgentProfile{}) == "bash");
}

TEST_CASE("harness commands pass the prompt by environment variable", "[agent]") {
    CHECK(agent_profile_build_cmd(agent_profile_from_name_or_cmd("claude")) ==
          "claude -p \"$SPACETIME_PROMPT\" --model \"claude-3-7-sonnet-20250219\" --dangerously-skip-permissions");
    CHECK(agent_profile_build_cmd(agent_profile_from_name_or_cmd("gemini")) ==
          "gemini --prompt \"$SPACETIME_PROMPT\" --model \"gemini-2.0-flash\" --yes --headless");
    CHECK(agent_profile_build_cmd(agent_profile_from_name_or_cmd("agy")) ==
          "agy --prompt \"$SPACETIME_PROMPT\" --model \"gemini-2.0-flash\" --yes");
    CHECK(agent_profile_build_cmd(agent_profile_from_name_or_cmd("codex")) ==
          "codex run --prompt \"$SPACETIME_PROMPT\" --model \"gpt-4o\"");
    CHECK(agent_profile_build_cmd(agent_profile_from_name_or_cmd("aider")) ==
          "aider --message \"$SPACETIME_PROMPT\" --yes --no-git");
}

TEST_CASE("the model flag is omitted when no model is set", "[agent]") {
    AgentProfile claude = agent_profile_from_name_or_cmd("claude");
    claude.model.clear();
    CHECK(agent_profile_build_cmd(claude) == "claude -p \"$SPACETIME_PROMPT\" --dangerously-skip-permissions");

    AgentProfile codex = agent_profile_from_name_or_cmd("codex");
    codex.model.clear();
    CHECK(agent_profile_build_cmd(codex) == "codex run --prompt \"$SPACETIME_PROMPT\"");
}

TEST_CASE("the agent environment carries the prompt and provider keys", "[agent]") {
    const ScopedEnv anthropic("ANTHROPIC_API_KEY", "sk-test-anthropic");
    const ScopedEnv openai("OPENAI_API_KEY", nullptr);
    const ScopedEnv unrelated("SPACETIME_UNRELATED", "nope");

    const auto env = agent_profile_get_env("fix the server");
    REQUIRE(env.size() >= 3);
    CHECK(env[0] == "SPACETIME_PROMPT=fix the server");
    CHECK(env[1] == "PROMPT=fix the server");
    CHECK(contains(env, "ANTHROPIC_API_KEY=sk-test-anthropic"));
    CHECK(std::none_of(env.begin(), env.end(), [](const std::string& e) {
        return e.starts_with("OPENAI_API_KEY=") || e.starts_with("SPACETIME_UNRELATED=");
    }));
}

TEST_CASE("long prompts are not truncated in the environment", "[agent]") {
    const std::string prompt(10'000, 'q');
    const auto env = agent_profile_get_env(prompt);
    REQUIRE(env.size() >= 2);
    CHECK(env[0] == "SPACETIME_PROMPT=" + prompt);
    CHECK(env[1] == "PROMPT=" + prompt);
}
