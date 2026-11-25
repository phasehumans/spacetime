#include "catch2/catch_amalgamated.hpp"

#include <string>

#include "task.hpp"
#include "test_helpers.hpp"

using namespace spacetime;

namespace {

void write_task(const TempDir& dir, const std::string& folder, const std::string& meta,
                const std::string& prompt = "do the thing\n") {
    dir.write(folder + "/prompt.txt", prompt);
    dir.write(folder + "/setup.sh", "true\n");
    dir.write(folder + "/test.sh", "true\n");
    if (!meta.empty()) dir.write(folder + "/meta.sh", meta);
}

} // namespace

TEST_CASE("meta.sh fields are parsed", "[task]") {
    TempDir dir;
    write_task(dir, "001-alpha",
               "# a comment\n"
               "TASK_ID=\"alpha-id\"\n"
               "  task_name = 'Alpha Task'  \r\n"
               "BASE_IMAGE=debian:12\n"
               "MAX_TURNS=42\n"
               "TIMEOUT_SECS=\"90\"\n"
               "DESCRIPTION=\"has = sign inside\"\n"
               "UNKNOWN_KEY=ignored\n"
               "not a key value line\n");

    const auto tasks = load_all_tasks(dir.path());
    REQUIRE(tasks);
    REQUIRE(tasks->size() == 1);

    const BenchmarkTask& t = tasks->front();
    CHECK(t.id == "alpha-id");
    CHECK(t.name == "Alpha Task");
    CHECK(t.base_image == "debian:12");
    CHECK(t.max_turns == 42);
    CHECK(t.timeout_secs == 90);
    CHECK(t.description == "has = sign inside");
    CHECK(t.prompt == "do the thing");
    CHECK(t.task_dir == dir.path() + "/001-alpha");
    CHECK(t.setup_script == dir.path() + "/001-alpha/setup.sh");
    CHECK(t.test_script == dir.path() + "/001-alpha/test.sh");
}

TEST_CASE("tasks without meta.sh fall back to defaults", "[task]") {
    TempDir dir;
    write_task(dir, "plain-task", "");

    const auto tasks = load_all_tasks(dir.path());
    REQUIRE(tasks);
    REQUIRE(tasks->size() == 1);

    const BenchmarkTask& t = tasks->front();
    CHECK(t.id == "plain-task");
    CHECK(t.name == "plain-task");
    CHECK(t.base_image == "ubuntu:22.04");
    CHECK(t.max_turns == 15);
    CHECK(t.timeout_secs == 30);
    CHECK(t.description.empty());
}

TEST_CASE("meta.toml is read when meta.sh is missing", "[task]") {
    TempDir dir;
    write_task(dir, "toml-task", "");
    dir.write("toml-task/meta.toml", "name = \"From Toml\"\ntimeout = 12\n");

    const auto tasks = load_all_tasks(dir.path());
    REQUIRE(tasks);
    REQUIRE(tasks->size() == 1);
    CHECK(tasks->front().name == "From Toml");
    CHECK(tasks->front().timeout_secs == 12);
}

TEST_CASE("incomplete task directories are skipped and results are sorted", "[task]") {
    TempDir dir;
    write_task(dir, "zeta", "");
    write_task(dir, "alpha", "");
    write_task(dir, "mid", "");
    dir.write("no-setup/prompt.txt", "x");
    dir.write("no-setup/test.sh", "true");
    dir.write("no-test/prompt.txt", "x");
    dir.write("no-test/setup.sh", "true");
    dir.write("not-a-task/readme.md", "x");
    dir.write("stray-file.txt", "x");

    const auto tasks = load_all_tasks(dir.path());
    REQUIRE(tasks);
    REQUIRE(tasks->size() == 3);
    CHECK((*tasks)[0].id == "alpha");
    CHECK((*tasks)[1].id == "mid");
    CHECK((*tasks)[2].id == "zeta");
}

TEST_CASE("a missing tasks directory is an error", "[task]") {
    TempDir dir;
    CHECK_FALSE(load_all_tasks(dir.path() + "/does-not-exist").has_value());
    CHECK_FALSE(find_task_by_id(dir.path() + "/does-not-exist", "anything").has_value());
}

TEST_CASE("prompts of any length are loaded whole", "[task]") {
    TempDir dir;
    const std::string long_prompt(20'000, 'p');
    write_task(dir, "long-prompt", "", "\n  " + long_prompt + "  \n\n");

    const auto tasks = load_all_tasks(dir.path());
    REQUIRE(tasks);
    REQUIRE(tasks->size() == 1);
    CHECK(tasks->front().prompt == long_prompt);
}

TEST_CASE("find_task_by_id matches exact ids, substrings and names", "[task]") {
    TempDir dir;
    write_task(dir, "001-nginx-config", "TASK_NAME=\"Fix Nginx Configuration\"\n");
    write_task(dir, "002-find-file", "TASK_NAME=\"Find The File\"\n");
    write_task(dir, "010-nginx-ssl", "TASK_NAME=\"Nginx SSL\"\n");

    SECTION("exact id wins over an earlier substring match") {
        const auto task = find_task_by_id(dir.path(), "010-nginx-ssl");
        REQUIRE(task);
        CHECK(task->id == "010-nginx-ssl");
    }
    SECTION("substring of the id returns the first match in id order") {
        const auto task = find_task_by_id(dir.path(), "nginx");
        REQUIRE(task);
        CHECK(task->id == "001-nginx-config");
    }
    SECTION("numeric prefix") {
        const auto task = find_task_by_id(dir.path(), "002");
        REQUIRE(task);
        CHECK(task->id == "002-find-file");
    }
    SECTION("a leading task- prefix is ignored") {
        const auto task = find_task_by_id(dir.path(), "task-002");
        REQUIRE(task);
        CHECK(task->id == "002-find-file");
    }
    SECTION("names match case-insensitively") {
        const auto task = find_task_by_id(dir.path(), "the FILE");
        REQUIRE(task);
        CHECK(task->id == "002-find-file");
    }
    SECTION("no match") {
        CHECK_FALSE(find_task_by_id(dir.path(), "kubernetes").has_value());
    }
}

TEST_CASE("the bundled task suite loads", "[task]") {
    const auto tasks = load_all_tasks(SPACETIME_TASKS_DIR);
    REQUIRE(tasks);
    CHECK(tasks->size() == 50);

    const auto nginx = find_task_by_id(SPACETIME_TASKS_DIR, "001");
    REQUIRE(nginx);
    CHECK(nginx->id == "001-nginx-config");
    CHECK(nginx->name == "Fix Nginx Configuration");
    CHECK(nginx->timeout_secs == 30);
    CHECK_FALSE(nginx->prompt.empty());
}
