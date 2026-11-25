#include "docker.hpp"

#include <algorithm>
#include <array>
#include <cerrno>
#include <charconv>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <format>
#include <thread>
#include <utility>

#include <poll.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sys/un.h>
#include <sys/wait.h>
#include <unistd.h>

#include "nlohmann/json.hpp"
#include "util.hpp"

namespace spacetime::docker {

using json = nlohmann::json;
using Clock = std::chrono::steady_clock;

namespace {

// Owns a file descriptor and closes it on scope exit.
class UniqueFd {
public:
    explicit UniqueFd(int fd = -1) : fd_(fd) {}
    ~UniqueFd() { reset(); }

    UniqueFd(UniqueFd&& other) noexcept : fd_(std::exchange(other.fd_, -1)) {}
    UniqueFd& operator=(UniqueFd&& other) noexcept {
        if (this != &other) {
            reset();
            fd_ = std::exchange(other.fd_, -1);
        }
        return *this;
    }
    UniqueFd(const UniqueFd&) = delete;
    UniqueFd& operator=(const UniqueFd&) = delete;

    int get() const { return fd_; }
    explicit operator bool() const { return fd_ >= 0; }

    void reset() {
        if (fd_ >= 0) ::close(fd_);
        fd_ = -1;
    }

private:
    int fd_;
};

std::vector<std::string>& active_containers() {
    static std::vector<std::string> containers;
    return containers;
}

UniqueFd connect_socket() {
    UniqueFd fd(::socket(AF_UNIX, SOCK_STREAM, 0));
    if (!fd) return fd;

    sockaddr_un addr{};
    addr.sun_family = AF_UNIX;
    std::strncpy(addr.sun_path, kSockPath, sizeof(addr.sun_path) - 1);

    if (::connect(fd.get(), reinterpret_cast<const sockaddr*>(&addr), sizeof(addr)) != 0) {
        fd.reset();
    }
    return fd;
}

bool write_all(int fd, std::string_view data) {
    while (!data.empty()) {
        const ssize_t n = ::write(fd, data.data(), data.size());
        if (n < 0) {
            if (errno == EINTR) continue;
            return false;
        }
        data.remove_prefix(static_cast<std::size_t>(n));
    }
    return true;
}

// Reads until `len` bytes arrived or the stream ended; returns the bytes read.
std::size_t read_exact(int fd, char* buf, std::size_t len) {
    std::size_t got = 0;
    while (got < len) {
        const ssize_t n = ::read(fd, buf + got, len - got);
        if (n > 0) {
            got += static_cast<std::size_t>(n);
        } else if (n < 0 && errno == EINTR) {
            continue;
        } else {
            break;
        }
    }
    return got;
}

std::string read_all(int fd) {
    std::string out;
    std::array<char, 8192> buf;
    for (;;) {
        const ssize_t n = ::read(fd, buf.data(), buf.size());
        if (n > 0) {
            out.append(buf.data(), static_cast<std::size_t>(n));
        } else if (n < 0 && errno == EINTR) {
            continue;
        } else {
            break;
        }
    }
    return out;
}

std::string decode_chunked(std::string_view body) {
    std::string out;
    std::size_t pos = 0;
    while (pos < body.size()) {
        const std::size_t eol = body.find("\r\n", pos);
        if (eol == std::string_view::npos) break;

        std::size_t chunk_size = 0;
        const auto parsed = std::from_chars(body.data() + pos, body.data() + eol, chunk_size, 16);
        if (parsed.ec != std::errc() || chunk_size == 0) break;

        pos = eol + 2;
        chunk_size = std::min(chunk_size, body.size() - pos);
        out.append(body.substr(pos, chunk_size));
        pos += chunk_size;
        if (body.substr(pos, 2) == "\r\n") pos += 2;
    }
    return out;
}

// Serializes a request body. Invalid UTF-8 (possible in task scripts and
// prompts) is replaced rather than thrown on.
std::string to_wire(const json& body) {
    return body.dump(-1, ' ', false, json::error_handler_t::replace);
}

// Returns the string field `key` of a JSON object response, if present.
std::optional<std::string> json_string_field(std::string_view body, const char* key) {
    const json doc = json::parse(body, nullptr, false);
    if (!doc.is_object()) return std::nullopt;
    const auto it = doc.find(key);
    if (it == doc.end() || !it->is_string()) return std::nullopt;
    return it->get<std::string>();
}

std::string generate_container_name() {
    using namespace std::chrono;
    const auto now = system_clock::now().time_since_epoch();
    const auto secs = duration_cast<seconds>(now).count();
    const auto usecs = duration_cast<microseconds>(now).count() % 1'000'000;
    return std::format("spacetime-{:x}-{:04x}", secs, usecs & 0xffff);
}

// Creates an exec instance running `bash -c <cmd>` as root and returns its id.
std::optional<std::string> create_exec(const std::string& container_id, const std::string& cmd,
                                       const std::vector<std::string>& env_vars) {
    json body = {
        {"AttachStdout", true},
        {"AttachStderr", true},
        {"User", "root"},
        {"WorkingDir", "/root"},
        {"Cmd", json::array({"/bin/bash", "-c", cmd})},
    };
    if (!env_vars.empty()) body["Env"] = env_vars;

    const auto resp = request("POST", std::format("/containers/{}/exec", container_id), to_wire(body));
    if (!resp || resp->status_code != 201) return std::nullopt;
    return json_string_field(resp->body, "Id");
}

// Polls the exec instance until it stops running and returns its exit code, or
// -1 if it is still running after ~5 seconds.
int wait_exec_exit_code(const std::string& exec_id) {
    const std::string path = std::format("/exec/{}/json", exec_id);
    for (int retry = 0; retry < 100; ++retry) {
        const auto resp = request("GET", path);
        if (resp && resp->status_code == 200) {
            const json doc = json::parse(resp->body, nullptr, false);
            if (doc.is_object()) {
                const auto running = doc.find("Running");
                if (running != doc.end() && !(running->is_boolean() && running->get<bool>())) {
                    const auto code = doc.find("ExitCode");
                    return (code != doc.end() && code->is_number_integer()) ? code->get<int>() : 0;
                }
            }
        }
        std::this_thread::sleep_for(std::chrono::milliseconds(50));
    }
    return -1;
}

// Starts the exec over a hijacked connection and demultiplexes Docker's
// 8-byte framed stream into stdout / stderr.
std::optional<ExecutionResult> run_exec_hijack(const std::string& exec_id, std::uint64_t timeout_secs, bool silent) {
    UniqueFd fd = connect_socket();
    if (!fd) return std::nullopt;

    const std::string body = to_wire({{"Detach", false}, {"Tty", false}});
    const std::string req = std::format(
        "POST /exec/{}/start HTTP/1.1\r\n"
        "Host: localhost\r\n"
        "User-Agent: spacetime-cpp\r\n"
        "Content-Type: application/json\r\n"
        "Content-Length: {}\r\n"
        "Connection: Upgrade\r\n"
        "Upgrade: tcp\r\n\r\n{}",
        exec_id, body.size(), body);
    if (!write_all(fd.get(), req)) return std::nullopt;

    // Consume the HTTP response header (up to the blank line); the framed
    // stream follows it on the same connection.
    std::string http_header;
    while (http_header.size() < 4095 && !http_header.ends_with("\r\n\r\n")) {
        char c;
        if (read_exact(fd.get(), &c, 1) != 1) break;
        http_header.push_back(c);
    }

    ExecutionResult res;
    const auto start = Clock::now();
    const auto timeout = std::chrono::seconds(timeout_secs);
    std::string payload;

    for (;;) {
        pollfd pfd{};
        pfd.fd = fd.get();
        pfd.events = POLLIN;
        const int pr = ::poll(&pfd, 1, 100);
        if (pr < 0 && errno == EINTR) continue;

        if (timeout_secs > 0 && Clock::now() - start > timeout) {
            res.timed_out = true;
            break;
        }

        if (pr == 0) continue; // poll timeout tick
        if (pr < 0) break;

        // Drain readable data before honouring a hang-up: the daemon closes
        // the connection right after the last frame, so both can be reported
        // by the same poll.
        if (!(pfd.revents & POLLIN)) break;

        std::array<unsigned char, 8> hdr;
        if (read_exact(fd.get(), reinterpret_cast<char*>(hdr.data()), hdr.size()) < hdr.size()) {
            break; // stream ended
        }

        const unsigned char stream_type = hdr[0]; // 1 = stdout, 2 = stderr
        const std::uint32_t payload_len = (static_cast<std::uint32_t>(hdr[4]) << 24) |
                                          (static_cast<std::uint32_t>(hdr[5]) << 16) |
                                          (static_cast<std::uint32_t>(hdr[6]) << 8) |
                                          (static_cast<std::uint32_t>(hdr[7]));
        if (payload_len == 0) continue;

        payload.resize(payload_len);
        payload.resize(read_exact(fd.get(), payload.data(), payload_len));

        if (stream_type == 1 || stream_type == 2) {
            std::FILE* echo = stream_type == 1 ? stdout : stderr;
            if (!silent) {
                std::fwrite(payload.data(), 1, payload.size(), echo);
                std::fflush(echo);
            }
            (stream_type == 1 ? res.stdout_data : res.stderr_data) += payload;
        }
    }

    fd.reset();

    res.duration_ms = static_cast<std::uint64_t>(
        std::chrono::duration_cast<std::chrono::milliseconds>(Clock::now() - start).count());
    res.exit_code = wait_exec_exit_code(exec_id);
    return res;
}

} // namespace

HttpResponse parse_http_response(std::string_view raw) {
    HttpResponse resp;

    if (raw.starts_with("HTTP/1.")) {
        const std::size_t space = raw.find(' ');
        if (space != std::string_view::npos) {
            const std::string_view rest = raw.substr(space + 1);
            std::from_chars(rest.data(), rest.data() + rest.size(), resp.status_code);
        }
    }

    const std::size_t header_end = raw.find("\r\n\r\n");
    if (header_end == std::string_view::npos) {
        resp.body = raw;
        return resp;
    }

    const std::string_view headers = raw.substr(0, header_end);
    const std::string_view body = raw.substr(header_end + 4);
    if (icontains(headers, "transfer-encoding: chunked")) {
        resp.body = decode_chunked(body);
    } else {
        resp.body = body;
    }
    return resp;
}

std::optional<HttpResponse> request(std::string_view method, std::string_view path, std::string_view json_body) {
    UniqueFd fd = connect_socket();
    if (!fd) return std::nullopt;

    const std::string req = std::format(
        "{} {} HTTP/1.1\r\n"
        "Host: localhost\r\n"
        "User-Agent: spacetime-cpp\r\n"
        "Accept: application/json\r\n"
        "Content-Type: application/json\r\n"
        "Content-Length: {}\r\n"
        "Connection: close\r\n\r\n{}",
        method, path, json_body.size(), json_body);
    if (!write_all(fd.get(), req)) return std::nullopt;

    const std::string raw = read_all(fd.get());
    if (raw.empty()) return std::nullopt;
    return parse_http_response(raw);
}

bool check_daemon() {
    const auto resp = request("GET", "/version");
    return resp && resp->status_code == 200;
}

bool ensure_image(const std::string& image_tag, bool force_rebuild) {
    if (!force_rebuild) {
        const auto resp = request("GET", std::format("/images/{}/json", image_tag));
        if (resp && resp->status_code == 200) return true;
    }

    print("\033[33mSandbox image '{}' not found or rebuild requested. Building...\033[0m\n", image_tag);
    std::fflush(stdout);

    const char* dockerfile = "Dockerfile";
    if (::access("Dockerfile", F_OK) != 0 && ::access("Dockerfile.sandbox", F_OK) == 0) {
        dockerfile = "Dockerfile.sandbox";
    }

    const pid_t pid = ::fork();
    if (pid == 0) {
        ::execlp("docker", "docker", "build", "-t", image_tag.c_str(), "-f", dockerfile, ".",
                 static_cast<char*>(nullptr));
        ::_exit(127);
    }

    int status = 0;
    if (pid < 0 || ::waitpid(pid, &status, 0) < 0 || !WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        eprint("\033[31mFailed to build Docker sandbox image '{}'\033[0m\n", image_tag);
        return false;
    }
    print("\033[32mSandbox image '{}' built successfully\033[0m\n", image_tag);
    return true;
}

std::optional<std::string> create_sandbox(const std::string& image, const std::string& mount_dir) {
    json body = {
        {"Image", image},
        {"Tty", true},
        {"WorkingDir", "/home/agent"},
        {"Cmd", json::array({"/bin/sh", "-c", "sleep infinity"})},
    };
    if (!mount_dir.empty()) {
        body["HostConfig"]["Binds"] = json::array({mount_dir + ":/workspace:rw"});
    }

    const auto resp = request("POST", std::format("/containers/create?name={}", generate_container_name()),
                              to_wire(body));
    if (!resp || resp->status_code != 201) {
        eprint("Failed to create container (HTTP {}): {}\n", resp ? resp->status_code : 0,
               resp ? resp->body : std::string());
        return std::nullopt;
    }

    auto container_id = json_string_field(resp->body, "Id");
    if (container_id) register_active_container(*container_id);
    return container_id;
}

bool start_container(const std::string& container_id) {
    const auto resp = request("POST", std::format("/containers/{}/start", container_id));
    return resp && (resp->status_code == 204 || resp->status_code == 304);
}

bool remove_container(const std::string& container_id, bool force) {
    const auto resp = request("DELETE", std::format("/containers/{}?v=1&force={}", container_id, force));
    unregister_active_container(container_id);
    return resp && (resp->status_code == 204 || resp->status_code == 404);
}

bool terminate_agent_processes(const std::string& container_id) {
    return execute_script(container_id, "pkill -u agent -9 2>/dev/null || pkill -u 1000 -9 2>/dev/null || true", 5)
        .has_value();
}

std::optional<ExecutionResult> execute_script(const std::string& container_id, const std::string& script_content,
                                              std::uint64_t timeout_secs) {
    const auto exec_id = create_exec(container_id, script_content, {});
    if (!exec_id) return std::nullopt;
    return run_exec_hijack(*exec_id, timeout_secs, true);
}

std::optional<ExecutionResult> execute_agent(const std::string& container_id, const std::string& cmd,
                                             const std::vector<std::string>& env_vars, std::uint64_t timeout_secs,
                                             bool silent) {
    const auto exec_id = create_exec(container_id, cmd, env_vars);
    if (!exec_id) return std::nullopt;
    return run_exec_hijack(*exec_id, timeout_secs, silent);
}

void register_active_container(const std::string& container_id) {
    auto& containers = active_containers();
    if (std::find(containers.begin(), containers.end(), container_id) == containers.end()) {
        containers.push_back(container_id);
    }
}

void unregister_active_container(const std::string& container_id) {
    std::erase(active_containers(), container_id);
}

void cleanup_all_active_containers() {
    // remove_container() unregisters as it goes, so iterate over a snapshot.
    const std::vector<std::string> snapshot = active_containers();
    for (const std::string& container_id : snapshot) {
        remove_container(container_id, true);
    }
    active_containers().clear();
}

std::string scrub_secrets(std::string input) {
    static constexpr std::array<const char*, 12> kSecretKeys = {
        "ANTHROPIC_API_KEY",  "GEMINI_API_KEY",     "GOOGLE_API_KEY",     "OPENAI_API_KEY",
        "DEEPSEEK_API_KEY",   "GROQ_API_KEY",       "OPENROUTER_API_KEY", "DATABRICKS_API_KEY",
        "DEVIN_API_KEY",      "CURSOR_API_KEY",     "INFLECTION_API_KEY", "HF_TOKEN",
    };

    for (const char* key : kSecretKeys) {
        const char* value = std::getenv(key);
        if (!value) continue;
        const std::string_view secret(value);
        if (secret.size() < 4) continue;

        for (std::size_t pos = input.find(secret); pos != std::string::npos;
             pos = input.find(secret, pos + secret.size())) {
            input.replace(pos, secret.size(), secret.size(), '*');
        }
    }
    return input;
}

} // namespace spacetime::docker
