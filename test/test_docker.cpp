#include "catch2/catch_amalgamated.hpp"

#include <string>

#include "docker.hpp"
#include "test_helpers.hpp"

using namespace spacetime;

TEST_CASE("plain HTTP responses are parsed", "[docker][http]") {
    const auto resp = docker::parse_http_response("HTTP/1.1 201 Created\r\n"
                                                  "Content-Type: application/json\r\n"
                                                  "Content-Length: 12\r\n\r\n"
                                                  "{\"Id\":\"abc\"}");
    CHECK(resp.status_code == 201);
    CHECK(resp.body == "{\"Id\":\"abc\"}");
}

TEST_CASE("responses without a body are parsed", "[docker][http]") {
    const auto resp = docker::parse_http_response("HTTP/1.1 204 No Content\r\nServer: Docker\r\n\r\n");
    CHECK(resp.status_code == 204);
    CHECK(resp.body.empty());
}

TEST_CASE("chunked HTTP responses are reassembled", "[docker][http]") {
    const auto resp = docker::parse_http_response("HTTP/1.1 200 OK\r\n"
                                                  "transfer-encoding: CHUNKED\r\n\r\n"
                                                  "5\r\nhello\r\n"
                                                  "1;ext=1\r\n \r\n"
                                                  "a\r\nworld\r\n123\r\n"
                                                  "0\r\n\r\n");
    CHECK(resp.status_code == 200);
    CHECK(resp.body == "hello world\r\n123");
}

TEST_CASE("a truncated chunk does not read past the buffer", "[docker][http]") {
    const auto resp = docker::parse_http_response("HTTP/1.1 200 OK\r\n"
                                                  "Transfer-Encoding: chunked\r\n\r\n"
                                                  "ff\r\nshort");
    CHECK(resp.body == "short");
}

TEST_CASE("a body that mentions chunked encoding is left alone", "[docker][http]") {
    const std::string body = "docs: Transfer-Encoding: chunked\r\n5\r\nhello";
    const auto resp = docker::parse_http_response("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n" + body);
    CHECK(resp.body == body);
}

TEST_CASE("input without HTTP headers is returned as the body", "[docker][http]") {
    const auto resp = docker::parse_http_response("garbage");
    CHECK(resp.status_code == 0);
    CHECK(resp.body == "garbage");
}

TEST_CASE("secret values from the environment are masked", "[docker][scrub]") {
    const ScopedEnv anthropic("ANTHROPIC_API_KEY", "sk-ant-secret");
    const ScopedEnv openai("OPENAI_API_KEY", "abc"); // too short to be treated as a secret
    const ScopedEnv hf("HF_TOKEN", nullptr);

    CHECK(docker::scrub_secrets("key=sk-ant-secret, again sk-ant-secret!") ==
          "key=*************, again *************!");
    CHECK(docker::scrub_secrets("abc stays") == "abc stays");
    CHECK(docker::scrub_secrets("nothing to hide") == "nothing to hide");
    CHECK(docker::scrub_secrets("").empty());
}

// Opt-in integration test: `make test-docker` (or `spacetime_tests "[docker]"`).
// Needs a running Docker daemon and a local ubuntu:24.04 image.
TEST_CASE("the Docker client runs a script in a sandbox container", "[.][docker]") {
    REQUIRE(docker::check_daemon());

    const auto container_id = docker::create_sandbox("ubuntu:24.04");
    REQUIRE(container_id);

    // Remove the container even if an assertion below fails.
    struct Cleanup {
        const std::string& id;
        ~Cleanup() { docker::remove_container(id, true); }
    } cleanup{*container_id};

    REQUIRE(docker::start_container(*container_id));

    SECTION("stdout, stderr and the exit code are captured") {
        const auto res = docker::execute_script(
            *container_id, "echo 'hello from spacetime' && whoami && echo oops >&2 && exit 3", 10);
        REQUIRE(res);
        CHECK(res->exit_code == 3);
        CHECK(res->stdout_data == "hello from spacetime\nroot\n");
        CHECK(res->stderr_data == "oops\n");
        CHECK_FALSE(res->timed_out);
    }

    SECTION("environment variables reach the agent command") {
        const std::string prompt = "multi\nline \"quoted\" $prompt " + std::string(8000, 'x');
        const auto res = docker::execute_agent(*container_id, "printf '%s' \"$SPACETIME_PROMPT\"",
                                               {"SPACETIME_PROMPT=" + prompt}, 10, true);
        REQUIRE(res);
        CHECK(res->exit_code == 0);
        CHECK(res->stdout_data == prompt);
    }

    SECTION("large outputs are captured whole") {
        const auto res = docker::execute_script(*container_id, "head -c 1000000 /dev/zero | tr '\\0' 'a'", 30);
        REQUIRE(res);
        CHECK(res->exit_code == 0);
        CHECK(res->stdout_data == std::string(1'000'000, 'a'));
    }

    SECTION("a script that outlives its timeout is reported") {
        const auto res = docker::execute_script(*container_id, "sleep 30", 1);
        REQUIRE(res);
        CHECK(res->timed_out);
        CHECK(res->exit_code == -1);
    }

    SECTION("removal succeeds and is idempotent") {
        CHECK(docker::remove_container(*container_id, true));
        CHECK(docker::remove_container(*container_id, true));
    }
}
