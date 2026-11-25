#pragma once

#include <string>
#include <string_view>
#include <vector>

#include "types.hpp"

namespace spacetime {

// Resolves a harness name (claude, gemini, agy, codex, aider) to its profile.
// A non-empty `custom_cmd` wins; an unknown name is treated as a custom
// command.
AgentProfile agent_profile_from_name_or_cmd(std::string_view name_or_harness, std::string_view custom_cmd = {});

// The shell command run inside the sandbox. The prompt is not interpolated; it
// is passed through the SPACETIME_PROMPT environment variable.
std::string agent_profile_build_cmd(const AgentProfile& profile);

// `KEY=value` environment entries for the agent process: the prompt plus any
// provider API keys set in the host environment.
std::vector<std::string> agent_profile_get_env(std::string_view prompt);

} // namespace spacetime
