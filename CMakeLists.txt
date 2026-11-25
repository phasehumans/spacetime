cmake_minimum_required(VERSION 3.16)
project(spacetime LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

if(NOT CMAKE_BUILD_TYPE AND NOT CMAKE_CONFIGURATION_TYPES)
    set(CMAKE_BUILD_TYPE Release CACHE STRING "Build type" FORCE)
endif()

option(SPACETIME_BUILD_TESTS "Build the unit tests" ON)
option(SPACETIME_STATIC_RUNTIME "Statically link libstdc++ and libgcc (portable release binaries)" OFF)

set(SPACETIME_WARNINGS -Wall -Wextra -pedantic)

# Everything except main() lives in a library so the tests can link against it.
add_library(spacetime_core STATIC
    src/agent.cpp
    src/docker.cpp
    src/eval.cpp
    src/report.cpp
    src/sha256.cpp
    src/task.cpp
    src/taskset.cpp
    src/tui.cpp
    src/util.cpp
)
target_include_directories(spacetime_core PUBLIC include)
target_include_directories(spacetime_core SYSTEM PUBLIC vendor)
target_compile_options(spacetime_core PRIVATE ${SPACETIME_WARNINGS})

add_executable(spacetime src/main.cpp)
target_link_libraries(spacetime PRIVATE spacetime_core)
target_compile_options(spacetime PRIVATE ${SPACETIME_WARNINGS})
if(SPACETIME_STATIC_RUNTIME)
    target_link_options(spacetime PRIVATE -static-libstdc++ -static-libgcc)
endif()

install(TARGETS spacetime RUNTIME DESTINATION bin)

if(SPACETIME_BUILD_TESTS)
    enable_testing()

    add_library(catch2 STATIC vendor/catch2/catch_amalgamated.cpp)
    target_include_directories(catch2 SYSTEM PUBLIC vendor)

    add_executable(spacetime_tests
        test/test_agent.cpp
        test/test_docker.cpp
        test/test_report.cpp
        test/test_sha256.cpp
        test/test_task.cpp
        test/test_taskset.cpp
    )
    target_link_libraries(spacetime_tests PRIVATE spacetime_core catch2)
    target_compile_options(spacetime_tests PRIVATE ${SPACETIME_WARNINGS})
    target_compile_definitions(spacetime_tests PRIVATE
        SPACETIME_TASKS_DIR="${CMAKE_CURRENT_SOURCE_DIR}/tasks")

    add_test(NAME spacetime_tests COMMAND spacetime_tests)
endif()
