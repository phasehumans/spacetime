#pragma once

#include <cstdint>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

#include "types.hpp"

// Minimal Docker Engine API client speaking HTTP/1.1 over the local AF_UNIX
// socket.
namespace spacetime::docker {

inline constexpr char kSockPath[] = "/var/run/docker.sock";

struct HttpResponse {
    int status_code = 0;
    std::string body;
};

// Splits a raw HTTP/1.1 response into status code and (de-chunked) body.
HttpResponse parse_http_response(std::string_view raw);

// Performs one request on a fresh connection. Returns nullopt if the daemon
// cannot be reached or sends nothing back.
std::optional<HttpResponse> request(std::string_view method, std::string_view path, std::string_view json_body = {});

bool check_daemon();
bool ensure_image(const std::string& image_tag, bool force_rebuild);

// Creates (but does not start) a sandbox container and returns its id.
std::optional<std::string> create_sandbox(const std::string& image, const std::string& mount_dir = {});
bool start_container(const std::string& container_id);
bool remove_container(const std::string& container_id, bool force);
bool terminate_agent_processes(const std::string& container_id);

// Runs `bash -c <script>` as root inside the container and captures its
// output. A timeout of 0 means no limit. Returns nullopt if the exec could not
// be created or attached to.
std::optional<ExecutionResult> execute_script(const std::string& container_id, const std::string& script_content,
                                              std::uint64_t timeout_secs);

// Same as execute_script, with extra `KEY=value` environment entries and
// optional live streaming of the output to this process's stdout / stderr.
std::optional<ExecutionResult> execute_agent(const std::string& container_id, const std::string& cmd,
                                             const std::vector<std::string>& env_vars, std::uint64_t timeout_secs,
                                             bool silent);

// Containers created by this process are tracked so they can be removed on
// SIGINT / SIGTERM.
void register_active_container(const std::string& container_id);
void unregister_active_container(const std::string& container_id);
void cleanup_all_active_containers();

// Masks the values of known API key environment variables found in `input`.
std::string scrub_secrets(std::string input);

} // namespace spacetime::docker
