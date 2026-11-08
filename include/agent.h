#ifndef SPACETIME_AGENT_H
#define SPACETIME_AGENT_H

#include "types.h"
#include <stddef.h>

int agent_profile_from_name_or_cmd(const char *name_or_harness, const char *custom_cmd, AgentProfile *out_profile);
int agent_profile_build_cmd(const AgentProfile *profile, const char *prompt, char *out_cmd, size_t out_cmd_sz);
int agent_profile_get_env(const AgentProfile *profile, const char *prompt, char ***out_env, size_t *out_count);
void agent_profile_free_env(char **env, size_t count);

#endif /* SPACETIME_AGENT_H */
