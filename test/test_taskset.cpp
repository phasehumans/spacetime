#include "catch2/catch_amalgamated.hpp"

#include <string>

#include "sha256.hpp"
#include "task.hpp"
#include "taskset.hpp"
#include "test_helpers.hpp"

using namespace spacetime;

TEST_CASE("the bundled taskset digest is stable", "[taskset]") {
    const auto tasks = load_all_tasks(SPACETIME_TASKS_DIR);
    REQUIRE(tasks);
    REQUIRE(tasks->size() == 50);

    const TasksetIdentity id = taskset_compute(*tasks);
    CHECK(id.size == 50);
    CHECK(id.hash == "7ecd749976c413ce1f51daf4dd09ca9e987548468c4bcf40430aad44582f3f16");
    CHECK(id.id == "v1-50-7ecd7499");
}

TEST_CASE("the manifest lists one sorted digest row per task", "[taskset]") {
    TempDir dir;
    for (const char* folder : {"b-task", "a-task"}) {
        dir.write(std::string(folder) + "/prompt.txt", "prompt");
        dir.write(std::string(folder) + "/setup.sh", "setup");
        dir.write(std::string(folder) + "/test.sh", "test");
    }
    dir.write("a-task/meta.sh", "TASK_ID=\"renamed\"\n");

    const auto tasks = load_all_tasks(dir.path());
    REQUIRE(tasks);
    REQUIRE(tasks->size() == 2);

    const std::string prompt = sha256_hex("prompt");
    const std::string setup = sha256_hex("setup");
    const std::string test = sha256_hex("test");
    const std::string meta = sha256_hex("TASK_ID=\"renamed\"\n");

    // Rows are keyed by folder name (not task id); a missing file is "-".
    const std::string expected = "a-task:" + prompt + ":" + setup + ":" + test + ":" + meta + "\n" +
                                 "b-task:" + prompt + ":" + setup + ":" + test + ":-\n";
    CHECK(taskset_manifest(*tasks) == expected);

    const TasksetIdentity id = taskset_compute(*tasks);
    CHECK(id.size == 2);
    CHECK(id.hash == sha256_hex(expected));
    CHECK(id.id == "v1-2-" + id.hash.substr(0, 8));
}

TEST_CASE("changing any task file changes the taskset hash", "[taskset]") {
    TempDir dir;
    dir.write("only/prompt.txt", "prompt");
    dir.write("only/setup.sh", "setup");
    dir.write("only/test.sh", "test");

    const auto before = load_all_tasks(dir.path());
    REQUIRE(before);
    const std::string hash_before = taskset_compute(*before).hash;

    dir.write("only/test.sh", "test # edited");
    const auto after = load_all_tasks(dir.path());
    REQUIRE(after);
    CHECK(taskset_compute(*after).hash != hash_before);
}

TEST_CASE("an empty taskset has a well-defined identity", "[taskset]") {
    const TasksetIdentity id = taskset_compute({});
    CHECK(id.size == 0);
    CHECK(id.hash == sha256_hex(""));
    CHECK(id.id == "v1-0-e3b0c442");
}
